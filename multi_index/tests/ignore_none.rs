use multi_index::MultiIndex;

#[derive(MultiIndex, Clone)]
struct TestElement {
    #[multi_index(how = "hashed", unique, ignore_none)]
    field1: Option<usize>,
}

#[test]
fn test_insert_and_get() {
    let mut map = MultiIndexTestElement::default();
    let elem1 = TestElement { field1: Some(1) };
    map.insert(elem1);

    let elem1_ref = map.get_by_field1(&Some(1)).unwrap();
    assert_eq!(elem1_ref.field1, Some(1));
    assert_eq!(map.len(), 1);
}

#[test]
fn test_insert_and_get_none() {
    let mut map = MultiIndexTestElement::default();
    let elem1 = TestElement { field1: None };
    let elem1_idx = map.insert(elem1);

    let elem1_ref = map.get_by_field1(&None);
    assert_eq!(elem1_ref.is_some(), false);
    assert_eq!(map.len(), 1);
    assert!(map.get(elem1_idx).unwrap().field1.is_none());

    let elem2 = TestElement { field1: Some(1) };
    let elem2_idx = map.insert(elem2);

    let elem2_ref = map.get_by_field1(&Some(1));
    assert_eq!(elem2_ref.is_some(), true);
    assert_eq!(map.len(), 2);
    assert_eq!(map.get(elem2_idx).unwrap().field1, Some(1 as usize));

    map.remove(elem2_idx);
    assert_eq!(map.len(), 1);

    map.remove(elem1_idx);
    assert_eq!(map.len(), 0);
}
