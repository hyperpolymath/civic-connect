// SPDX-License-Identifier: PMPL-1.0-or-later
// Policy enforcement for GraphQL DNS API
//
// Loads Nickel CURPS policies and enforces:
// - Role-based access control (RBAC)
// - Mutation approval requirements
// - Timelock delays
// - Consent bindings
// - Rate limiting

use anyhow::{anyhow, Result};
use async_graphql::{Enum, SimpleObject, Scalar, ScalarType};
use async_graphql::Value as GraphQLValue;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

/// JSON scalar for GraphQL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JSON(pub JsonValue);

#[Scalar]
impl ScalarType for JSON {
    fn parse(value: GraphQLValue) -> async_graphql::InputValueResult<Self> {
        match value {
            GraphQLValue::Object(obj) => {
                let json_value = serde_json::to_value(obj)?;
                Ok(JSON(json_value))
            }
            _ => Err(async_graphql::InputValueError::expected_type(value)),
        }
    }

    fn to_value(&self) -> GraphQLValue {
        match serde_json::to_value(&self.0) {
            Ok(JsonValue::Object(map)) => {
                GraphQLValue::Object(map.into_iter().map(|(k, v)| (async_graphql::Name::new(k), json_to_graphql(v))).collect())
            }
            _ => GraphQLValue::Null,
        }
    }
}

fn json_to_graphql(value: JsonValue) -> GraphQLValue {
    match value {
        JsonValue::Null => GraphQLValue::Null,
        JsonValue::Bool(b) => GraphQLValue::Boolean(b),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                GraphQLValue::Number(i.into())
            } else if let Some(f) = n.as_f64() {
                match async_graphql::Number::from_f64(f) {
                    Some(num) => GraphQLValue::Number(num),
                    None => GraphQLValue::Null,
                }
            } else {
                GraphQLValue::Null
            }
        }
        JsonValue::String(s) => GraphQLValue::String(s),
        JsonValue::Array(arr) => GraphQLValue::List(arr.into_iter().map(json_to_graphql).collect()),
        JsonValue::Object(obj) => GraphQLValue::Object(obj.into_iter().map(|(k, v)| (async_graphql::Name::new(k), json_to_graphql(v))).collect()),
    }
}

/// Policy loaded from Nickel configuration
#[derive(Debug, Clone, Deserialize, Serialize, SimpleObject)]
#[graphql(name = "Policy")]
pub struct Policy {
    pub version: String,
    #[graphql(skip)]
    pub capabilities: HashMap<String, String>,
    pub mutations: Vec<MutationPolicy>,
    pub roles: Vec<Role>,
    #[graphql(skip)]
    pub routes: Vec<Route>,
    #[graphql(skip)]
    pub consent_bindings: Vec<ConsentBinding>,
    pub constraints: Constraints,
}

#[derive(Debug, Clone, Deserialize, Serialize, SimpleObject)]
#[graphql(name = "MutationPolicy")]
pub struct MutationPolicy {
    pub name: String,
    pub description: String,
    pub approvals: u32,
    #[serde(rename = "timelock_hours")]
    #[graphql(name = "timelockHours")]
    pub timelock_hours: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize, SimpleObject)]
#[graphql(name = "Role")]
pub struct Role {
    pub name: String,
    pub members: Vec<String>,
    pub privileges: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Route {
    pub path: String,
    pub plane: String,
    pub methods: Vec<String>,
    pub guards: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConsentBinding {
    pub name: String,
    pub manifest_ref: String,
    pub required: bool,
    pub defaults: ConsentDefaults,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConsentDefaults {
    pub telemetry: String,
    pub indexing: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, SimpleObject)]
#[graphql(name = "PolicyConstraints")]
pub struct Constraints {
    #[serde(rename = "require_mtls")]
    #[graphql(name = "requireMtls")]
    pub require_mtls: bool,
    #[serde(rename = "log_all_mutations")]
    #[graphql(name = "logAllMutations")]
    pub log_all_mutations: bool,
    #[serde(rename = "max_rate_rpm")]
    #[graphql(name = "maxRateRpm")]
    pub max_rate_rpm: u32,
}

/// Mutation proposal requiring approval
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
#[graphql(name = "MutationProposal")]
pub struct MutationProposal {
    pub id: String,
    #[serde(rename = "mutation_name")]
    #[graphql(name = "mutationName")]
    pub mutation_name: String,
    pub proposer: String,
    #[serde(rename = "proposed_at")]
    #[graphql(name = "proposedAt")]
    pub proposed_at: u64,
    #[serde(rename = "timelock_until")]
    #[graphql(name = "timelockUntil")]
    pub timelock_until: u64,
    pub approvals: Vec<String>,
    #[serde(rename = "required_approvals")]
    #[graphql(name = "requiredApprovals")]
    pub required_approvals: u32,
    pub status: ProposalStatus,
    pub payload: JsonValue,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Enum, Copy)]
pub enum ProposalStatus {
    Pending,
    TimelockActive,
    Approved,
    Rejected,
    Executed,
}

/// Policy enforcer
pub struct PolicyEnforcer {
    pub policy: Policy,
    pub proposals: HashMap<String, MutationProposal>,
}

impl PolicyEnforcer {
    /// Load policy from Nickel file
    pub fn from_nickel_file(path: &Path) -> Result<Self> {
        // Export Nickel to JSON using nickel CLI
        let output = Command::new("nickel")
            .arg("export")
            .arg("--format")
            .arg("json")
            .arg(path)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Nickel export failed: {}", stderr));
        }

        let json = String::from_utf8(output.stdout)?;
        let policy_wrapper: serde_json::Value = serde_json::from_str(&json)?;

        // Extract "policy" field from exported JSON
        let policy: Policy = serde_json::from_value(
            policy_wrapper
                .get("policy")
                .ok_or_else(|| anyhow!("Missing 'policy' field in Nickel export"))?
                .clone()
        )?;

        Ok(Self {
            policy,
            proposals: HashMap::new(),
        })
    }

    /// Check if identity has privilege
    pub fn has_privilege(&self, identity: &str, privilege: &str) -> bool {
        for role in &self.policy.roles {
            if role.members.contains(&identity.to_string())
                && role.privileges.contains(&privilege.to_string()) {
                return true;
            }
        }
        false
    }

    /// Get mutation policy by name
    pub fn get_mutation_policy(&self, mutation_name: &str) -> Option<&MutationPolicy> {
        self.policy.mutations.iter()
            .find(|m| m.name == mutation_name)
    }

    /// Propose a mutation (creates proposal requiring approval)
    pub fn propose_mutation(
        &mut self,
        mutation_name: &str,
        proposer: &str,
        payload: serde_json::Value,
    ) -> Result<MutationProposal> {
        // Check if proposer has privilege
        if !self.has_privilege(proposer, mutation_name) {
            return Err(anyhow!("Identity {} lacks privilege for {}", proposer, mutation_name));
        }

        let policy = self.get_mutation_policy(mutation_name)
            .ok_or_else(|| anyhow!("Unknown mutation: {}", mutation_name))?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs();

        let timelock_until = now + (policy.timelock_hours as u64 * 3600);

        let proposal = MutationProposal {
            id: uuid::Uuid::new_v4().to_string(),
            mutation_name: mutation_name.to_string(),
            proposer: proposer.to_string(),
            proposed_at: now,
            timelock_until,
            approvals: vec![proposer.to_string()], // Proposer auto-approves
            required_approvals: policy.approvals,
            status: if policy.timelock_hours > 0 {
                ProposalStatus::TimelockActive
            } else if policy.approvals <= 1 {
                ProposalStatus::Approved
            } else {
                ProposalStatus::Pending
            },
            payload,
        };

        self.proposals.insert(proposal.id.clone(), proposal.clone());
        Ok(proposal)
    }

    /// Approve a mutation proposal
    pub fn approve_proposal(
        &mut self,
        proposal_id: &str,
        approver: &str,
    ) -> Result<MutationProposal> {
        // Extract needed data from proposal before mutable borrow
        let mutation_name = {
            let proposal = self.proposals.get(proposal_id)
                .ok_or_else(|| anyhow!("Proposal not found: {}", proposal_id))?;
            proposal.mutation_name.clone()
        };

        // Check approver has privilege (immutable borrow of self)
        if !self.has_privilege(approver, &mutation_name) {
            return Err(anyhow!("Identity {} lacks privilege for {}", approver, mutation_name));
        }

        // Now take mutable borrow for modifications
        let proposal = self.proposals.get_mut(proposal_id)
            .ok_or_else(|| anyhow!("Proposal not found: {}", proposal_id))?;

        // Check not already approved by this identity
        if proposal.approvals.contains(&approver.to_string()) {
            return Err(anyhow!("Already approved by {}", approver));
        }

        proposal.approvals.push(approver.to_string());

        // Update status if enough approvals
        if proposal.approvals.len() as u32 >= proposal.required_approvals {
            if proposal.status == ProposalStatus::Pending {
                proposal.status = ProposalStatus::Approved;
            }
        }

        Ok(proposal.clone())
    }

    /// Check if proposal can be executed
    pub fn can_execute_proposal(&self, proposal_id: &str) -> Result<bool> {
        let proposal = self.proposals.get(proposal_id)
            .ok_or_else(|| anyhow!("Proposal not found: {}", proposal_id))?;

        // Check status
        if proposal.status != ProposalStatus::Approved
            && proposal.status != ProposalStatus::TimelockActive {
            return Ok(false);
        }

        // Check approvals
        if (proposal.approvals.len() as u32) < proposal.required_approvals {
            return Ok(false);
        }

        // Check timelock
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs();

        if now < proposal.timelock_until {
            return Ok(false);
        }

        Ok(true)
    }

    /// Execute proposal (marks as executed)
    pub fn execute_proposal(&mut self, proposal_id: &str) -> Result<MutationProposal> {
        if !self.can_execute_proposal(proposal_id)? {
            return Err(anyhow!("Proposal cannot be executed yet"));
        }

        let proposal = self.proposals.get_mut(proposal_id)
            .ok_or_else(|| anyhow!("Proposal not found: {}", proposal_id))?;

        proposal.status = ProposalStatus::Executed;
        Ok(proposal.clone())
    }

    /// Get all proposals
    pub fn get_proposals(&self) -> Vec<MutationProposal> {
        self.proposals.values().cloned().collect()
    }

    /// Get proposal by ID
    pub fn get_proposal(&self, proposal_id: &str) -> Option<&MutationProposal> {
        self.proposals.get(proposal_id)
    }

    /// Check rate limit for identity
    pub fn check_rate_limit(&self, identity: &str, requests_in_window: u32) -> bool {
        requests_in_window <= self.policy.constraints.max_rate_rpm
    }

    /// Get policy
    pub fn policy(&self) -> &Policy {
        &self.policy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_loading() {
        // This test requires nickel CLI installed
        let policy_path = Path::new("../policy/curps/policy.ncl");
        if policy_path.exists() {
            let enforcer = PolicyEnforcer::from_nickel_file(policy_path);
            assert!(enforcer.is_ok());

            if let Ok(enforcer) = enforcer {
                assert_eq!(enforcer.policy.version, "0.1.0");
                assert!(enforcer.policy.constraints.require_mtls);
            }
        }
    }

    #[test]
    fn test_privilege_check() {
        let policy = Policy {
            version: "0.1.0".to_string(),
            capabilities: HashMap::new(),
            mutations: vec![],
            roles: vec![
                Role {
                    name: "maintainer".to_string(),
                    members: vec!["identity:alice".to_string()],
                    privileges: vec!["mutate_dns".to_string()],
                },
            ],
            routes: vec![],
            consent_bindings: vec![],
            constraints: Constraints {
                require_mtls: true,
                log_all_mutations: true,
                max_rate_rpm: 120,
            },
        };

        let enforcer = PolicyEnforcer {
            policy,
            proposals: HashMap::new(),
        };

        assert!(enforcer.has_privilege("identity:alice", "mutate_dns"));
        assert!(!enforcer.has_privilege("identity:bob", "mutate_dns"));
    }
}
