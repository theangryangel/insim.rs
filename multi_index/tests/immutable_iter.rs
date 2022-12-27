use multi_index::MultiIndex;

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
struct TestNonPrimitiveType(u64);

#[derive(MultiIndex, Clone, Debug)]
#[multi_index(rename = "IndexedTestElement")]
struct TestElement {
    #[multi_index(how = "hashed")]
    field1: TestNonPrimitiveType,
    #[multi_index(how = "ordered", unique)]
    field3: usize,
}

#[test]
fn test_immutable_iter_by_field1() {
    let mut map = IndexedTestElement::default();
    let elem1 = TestElement {
        field1: TestNonPrimitiveType(42),
        field3: 1,
    };
    let elem2 = TestElement {
        field1: TestNonPrimitiveType(42),
        field3: 0,
    };

    map.insert(elem2);
    map.insert(elem1);

    let map = map.clone();

    for (idx, elem) in map.iter_by_field1().enumerate() {
        // Elements remain in inserted order when they have a non_unique key
        assert_eq!(idx, elem.field3);
    }
}
