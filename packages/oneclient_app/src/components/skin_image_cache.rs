use bytes::Bytes;

/// A render cache is not reactive state. Changing it must not schedule another
/// render, and equal PNG data must remain a hit after a fresh allocation.
pub(super) struct SkinImageCache<T> {
    source: Option<Bytes>,
    decoded: Option<T>,
}

impl<T> Default for SkinImageCache<T> {
    fn default() -> Self {
        Self {
            source: None,
            decoded: None,
        }
    }
}

impl<T: Clone> SkinImageCache<T> {
    pub fn get_or_decode(
        &mut self,
        source: &Bytes,
        decode: impl FnOnce(&Bytes) -> Option<T>,
    ) -> Option<T> {
        if self.source.as_ref() != Some(source) {
            self.decoded = decode(source);
            self.source = Some(source.clone());
        }
        self.decoded.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_skin_data_from_new_allocations_does_not_decode_again() {
        let mut cache = SkinImageCache::default();
        let mut decodes = 0;
        for _ in 0..100 {
            let png = Bytes::copy_from_slice(b"unchanged PNG data");
            assert_eq!(
                cache.get_or_decode(&png, |_| {
                    decodes += 1;
                    Some(42)
                }),
                Some(42)
            );
        }
        assert_eq!(decodes, 1);
    }

    #[test]
    fn changing_the_skin_refreshes_the_cached_image() {
        let mut cache = SkinImageCache::default();
        let first = Bytes::from_static(b"first PNG");
        let second = Bytes::from_static(b"second PNG");
        assert_eq!(cache.get_or_decode(&first, |_| Some(1)), Some(1));
        assert_eq!(cache.get_or_decode(&second, |_| Some(2)), Some(2));
        assert_eq!(
            cache.get_or_decode(&second, |_| panic!("unchanged skin was decoded again")),
            Some(2)
        );
    }

    #[test]
    fn invalid_image_is_cached_until_the_input_changes() {
        let mut cache = SkinImageCache::<u32>::default();
        let broken = Bytes::from_static(b"broken PNG");
        assert_eq!(cache.get_or_decode(&broken, |_| None), None);
        assert_eq!(
            cache.get_or_decode(&broken, |_| panic!("broken skin was decoded again")),
            None
        );
        assert_eq!(
            cache.get_or_decode(&Bytes::from_static(b"valid PNG"), |_| Some(7)),
            Some(7)
        );
    }
}
