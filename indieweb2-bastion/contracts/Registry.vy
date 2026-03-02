# SPDX-License-Identifier: MIT
# DNS record registry for on-chain provenance anchoring.
# @version ^0.3.10

owner: public(address)
records: public(HashMap[address, String[128]])

event RecordSet:
    addr: indexed(address)
    record: String[128]

event OwnershipTransferred:
    previousOwner: indexed(address)
    newOwner: indexed(address)

@external
def __init__():
    self.owner = msg.sender

@external
def set_record(addr: address, record: String[128]):
    assert msg.sender == self.owner, "Only owner"
    self.records[addr] = record
    log RecordSet(addr, record)

@external
def transfer_ownership(new_owner: address):
    assert msg.sender == self.owner, "Only owner"
    assert new_owner != empty(address), "Zero address"
    log OwnershipTransferred(self.owner, new_owner)
    self.owner = new_owner

@view
@external
def get_record(addr: address) -> String[128]:
    return self.records[addr]
