use indexed_slab::IndexedSlab;

#[derive(IndexedSlab, Clone)]
pub struct MultipleOrderedNonUniqueStruct {
    #[indexed_slab(how = "ordered")]
    field1: u32,
    #[indexed_slab(how = "ordered")]
    field2: u64,
}

#[test]
fn test_remove_ordered_non_unique_field1_get_ordered_non_unique_field2() {
    let mut map = IndexedSlabMultipleOrderedNonUniqueStruct::default();

    map.insert(MultipleOrderedNonUniqueStruct {
        field1: 1,
        field2: 999,
    });
    map.insert(MultipleOrderedNonUniqueStruct {
        field1: 2,
        field2: 999,
    });

    let a = map.remove_by_field1(&1);
    let b = map.get_by_field2(&999);

    assert_eq!(a.len(), 1);
    assert_eq!(b.len(), 1);
}

#[test]
fn test_remove_ordered_non_unique_field2_get_ordered_non_unique_field1() {
    let mut map = IndexedSlabMultipleOrderedNonUniqueStruct::default();

    map.insert(MultipleOrderedNonUniqueStruct {
        field1: 1,
        field2: 999,
    });
    map.insert(MultipleOrderedNonUniqueStruct {
        field1: 2,
        field2: 999,
    });

    let a = map.remove_by_field2(&999);
    let b = map.get_by_field1(&1);
    let c = map.get_by_field1(&2);

    assert_eq!(a.len(), 2);
    assert_eq!(b.len(), 0);
    assert_eq!(c.len(), 0);
}

#[derive(IndexedSlab, Clone)]
pub struct OrderedNonUniqueAndHashedNonUniqueStruct {
    #[indexed_slab(how = "hashed")]
    field1: u32,
    #[indexed_slab(how = "ordered")]
    field2: u64,
}

#[test]
fn test_remove_hashed_non_unique_field1_get_ordered_non_unique_field2() {
    let mut map = IndexedSlabOrderedNonUniqueAndHashedNonUniqueStruct::default();

    map.insert(OrderedNonUniqueAndHashedNonUniqueStruct {
        field1: 1,
        field2: 999,
    });
    map.insert(OrderedNonUniqueAndHashedNonUniqueStruct {
        field1: 2,
        field2: 999,
    });

    let a = map.remove_by_field1(&1);
    let b = map.get_by_field2(&999);

    assert_eq!(a.len(), 1);
    assert_eq!(b.len(), 1);
}

#[test]
fn test_remove_ordered_non_unique_field2_get_hashed_non_unique_field1() {
    let mut map = IndexedSlabOrderedNonUniqueAndHashedNonUniqueStruct::default();

    map.insert(OrderedNonUniqueAndHashedNonUniqueStruct {
        field1: 1,
        field2: 999,
    });
    map.insert(OrderedNonUniqueAndHashedNonUniqueStruct {
        field1: 2,
        field2: 999,
    });

    let a = map.remove_by_field2(&999);
    let b = map.get_by_field1(&1);
    let c = map.get_by_field1(&2);

    assert_eq!(a.len(), 2);
    assert_eq!(b.len(), 0);
    assert_eq!(c.len(), 0);
}

#[derive(IndexedSlab, Clone)]
pub struct MultipleHashedNonUniqueStruct {
    #[indexed_slab(how = "hashed")]
    field1: u32,
    #[indexed_slab(how = "ordered")]
    field2: u64,
}

#[test]
fn test_remove_hashed_non_unique_field1_get_hashed_non_unique_field2() {
    let mut map = IndexedSlabMultipleHashedNonUniqueStruct::default();

    map.insert(MultipleHashedNonUniqueStruct {
        field1: 1,
        field2: 999,
    });
    map.insert(MultipleHashedNonUniqueStruct {
        field1: 2,
        field2: 999,
    });

    let a = map.remove_by_field1(&1);
    let b = map.get_by_field2(&999);

    assert_eq!(a.len(), 1);
    assert_eq!(b.len(), 1);
}

#[test]
fn test_remove_hashed_non_unique_field2_get_hashed_non_unique_field1() {
    let mut map = IndexedSlabMultipleHashedNonUniqueStruct::default();

    map.insert(MultipleHashedNonUniqueStruct {
        field1: 1,
        field2: 999,
    });
    map.insert(MultipleHashedNonUniqueStruct {
        field1: 2,
        field2: 999,
    });

    let a = map.remove_by_field2(&999);
    let b = map.get_by_field1(&1);
    let c = map.get_by_field1(&2);

    assert_eq!(a.len(), 2);
    assert_eq!(b.len(), 0);
    assert_eq!(c.len(), 0);
}

#[test]
fn test_clear() {
    let mut map = IndexedSlabMultipleOrderedNonUniqueStruct::default();

    map.insert(MultipleOrderedNonUniqueStruct {
        field1: 1,
        field2: 999,
    });
    map.insert(MultipleOrderedNonUniqueStruct {
        field1: 2,
        field2: 999,
    });
    assert_eq!(map.len(), 2);

    map.clear();
    assert!(map.is_empty());

    let a = map.remove_by_field2(&999);
    let b = map.remove_by_field1(&1);
    let c = map.remove_by_field1(&2);
    assert!(a.is_empty());
    assert!(b.is_empty());
    assert!(c.is_empty());
}
