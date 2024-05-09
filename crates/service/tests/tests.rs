use service::add;

#[test]
fn test_addition() -> anyhow::Result<()> {
    let sum = add(2, 2);
    assert_eq!(sum, 4);
    Ok(())
}
