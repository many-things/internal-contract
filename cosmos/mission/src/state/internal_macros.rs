#[macro_export]
macro_rules! snapshot_item {
    ($storage_key: expr, $strategy: expr) => {
        cw_storage_plus::SnapshotItem::new(
            $storage_key,
            concat!($storage_key, "__checkpoints"),
            concat!($storage_key, "__changelog"),
            $strategy,
        )
    };
}
