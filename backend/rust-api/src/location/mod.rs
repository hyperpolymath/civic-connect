// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2025 Jonathan D.A. Jewell

//! Location services using H3 hexagonal grid
//!
//! Privacy design:
//! - Server NEVER receives or stores exact coordinates
//! - Client computes H3 cell ID locally
//! - Server only works with cell IDs
//!
//! H3 resolution 7 = ~5km hexagon diameter
//! Good balance of privacy vs. discovery usefulness

use h3o::{CellIndex, Resolution};

/// H3 resolution for location storage
/// Resolution 7 = approximately 5km hexagon diameter
pub const LOCATION_RESOLUTION: Resolution = Resolution::Seven;

/// Validate H3 cell ID format
pub fn is_valid_cell(cell_str: &str) -> bool {
    if cell_str.len() != 15 {
        return false;
    }

    cell_str.parse::<CellIndex>().is_ok()
}

/// Get neighboring cells within N rings
/// Ring 0 = just the cell itself
/// Ring 1 = cell + 6 immediate neighbors
/// Ring 2 = cell + 6 neighbors + 12 outer neighbors
pub fn get_neighbors(cell_str: &str, rings: u32) -> Vec<String> {
    let cell = match cell_str.parse::<CellIndex>() {
        Ok(c) => c,
        Err(_) => return vec![cell_str.to_string()],
    };

    // Collect cells within the specified number of rings
    let mut cells = vec![cell_str.to_string()];

    if rings > 0 {
        // grid_disk returns cells within the given distance
        for neighbor in cell.grid_disk::<Vec<_>>(rings) {
            let neighbor_str = neighbor.to_string();
            if neighbor_str != cell_str {
                cells.push(neighbor_str);
            }
        }
    }

    cells
}

/// Calculate approximate distance between two cells in kilometers
/// This is a rough estimate based on cell center distance
pub fn approximate_distance_km(cell1: &str, cell2: &str) -> Option<f64> {
    let c1 = cell1.parse::<CellIndex>().ok()?;
    let c2 = cell2.parse::<CellIndex>().ok()?;

    let ll1 = c1.to_lat_lng();
    let ll2 = c2.to_lat_lng();

    // Haversine formula
    let r = 6371.0; // Earth's radius in km

    let lat1 = ll1.lat().to_radians();
    let lat2 = ll2.lat().to_radians();
    let dlat = (ll2.lat() - ll1.lat()).to_radians();
    let dlon = (ll2.lng() - ll1.lng()).to_radians();

    let a = (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().asin();

    Some(r * c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_cell_format() {
        // Valid H3 index (resolution 7)
        assert!(is_valid_cell("872830828ffffff"));

        // Invalid - wrong length
        assert!(!is_valid_cell("123456"));

        // Invalid - not hex
        assert!(!is_valid_cell("zzzzzzzzzzzzzzzz"));
    }

    #[test]
    fn test_get_neighbors() {
        let cell = "872830828ffffff";

        // Ring 0 should just return the cell
        let ring0 = get_neighbors(cell, 0);
        assert_eq!(ring0.len(), 1);
        assert_eq!(ring0[0], cell);

        // Ring 1 should return 7 cells (center + 6 neighbors)
        let ring1 = get_neighbors(cell, 1);
        assert_eq!(ring1.len(), 7);
        assert!(ring1.contains(&cell.to_string()));
    }
}
