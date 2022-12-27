use multi_index::MultiIndex;

#[derive(Hash, PartialEq, Eq, Clone, Debug, PartialOrd, Ord)]
struct TestNonPrimitiveType(u64);

#[derive(MultiIndex, Clone, Debug)]
struct TestElement {
    #[multi_index(how = "ordered")]
    field1: TestNonPrimitiveType,
    #[multi_index(how = "ordered", unique)]
    field3: usize,
}

#[test]
fn test_insert_and_get_by_field1() {
    let mut map = MultiIndexTestElement::default();
    let elem1 = TestElement {
        field1: TestNonPrimitiveType(42),
        field3: 0,
    };
    let elem2 = TestElement {
        field1: TestNonPrimitiveType(42),
        field3: 1,
    };

    map.insert(elem2);
    map.insert(elem1);

    let elems = map.get_by_field1(&TestNonPrimitiveType(42));
    assert_eq!(elems.len(), 2);
    assert_eq!(map.len(), 2);
}

#[test]
fn test_insert_and_remove_by_field1() {
    let mut map = MultiIndexTestElement::default();
    let elem1 = TestElement {
        field1: TestNonPrimitiveType(42),
        field3: 0,
    };
    let elem2 = TestElement {
        field1: TestNonPrimitiveType(42),
        field3: 1,
    };

    map.insert(elem2);
    map.insert(elem1);

    let elems = map.remove_by_field1(&TestNonPrimitiveType(42));
    assert_eq!(elems.len(), 2);
    assert!(map.is_empty());
}

#[test]
fn test_insert_and_modify_by_field3_and_get_by_field1() {
    let mut map = MultiIndexTestElement::default();
    let elem1 = TestElement {
        field1: TestNonPrimitiveType(42),
        field3: 0,
    };
    let elem2 = TestElement {
        field1: TestNonPrimitiveType(42),
        field3: 1,
    };

    map.insert(elem2);
    map.insert(elem1);

    map.modify_by_field3(&0, |e| e.field1 = TestNonPrimitiveType(43));

    let elems = map.get_by_field1(&TestNonPrimitiveType(42));
    assert_eq!(elems.len(), 1);
    assert_eq!(map.len(), 2);
}

#[test]
fn test_insert_and_modify_by_field3_and_remove_by_field1() {
    let mut map = MultiIndexTestElement::default();
    let elem1 = TestElement {
        field1: TestNonPrimitiveType(42),
        field3: 0,
    };
    let elem2 = TestElement {
        field1: TestNonPrimitiveType(42),
        field3: 1,
    };

    map.insert(elem2);
    map.insert(elem1);

    map.modify_by_field3(&0, |e| e.field1 = TestNonPrimitiveType(43));

    let elems = map.remove_by_field1(&TestNonPrimitiveType(43));
    assert_eq!(elems.len(), 1);
    assert_eq!(map.len(), 1);
}

#[test]
fn test_insert_and_iter_by_field1() {
    let mut map = MultiIndexTestElement::default();
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

    for (idx, elem) in map.iter_by_field1().enumerate() {
        // Elements remain in inserted order when they have a non_unique key
        assert_eq!(idx, elem.field3);
    }
}
