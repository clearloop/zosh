//! Tests for the initialize instruction

use mollusk_svm::Mollusk;

#[test]
fn test_initialize_success() {
    let mollusk = Mollusk::new(&zyphers::ID, "zyphers");
}
