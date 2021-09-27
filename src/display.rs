use std::mem::size_of;

// Default mode graphics
const GFX_WIDTH_DEFAULT: usize = 64;
const GFX_HEIGHT_DEFAULT: usize = 32;

type Storage = u32;
const STORAGE_BITS: usize = Storage::BITS as usize;
const PACKED_WIDTH: usize = GFX_WIDTH_DEFAULT / STORAGE_BITS;

struct Bounds {
    pub min: usize,
    pub max: usize,
}

impl Bounds {
    pub fn new(initial: usize) -> Self {
        Self {
            min: initial,
            max: initial,
        }
    }

    pub fn extend(&mut self, value: usize) {
        if self.min > value {
            self.min = value;
        } else if self.max < value {
            self.max = value;
        }
    }
}

pub struct ModificationData<'a> {
    pub offset: usize,
    pub data: &'a [Storage],
}

pub struct DisplayState {
    packed_state: [Storage; PACKED_WIDTH * GFX_HEIGHT_DEFAULT],
    was_modified: bool,
    modification: Bounds,
}

impl DisplayState {
    pub fn new() -> Self {
        Self {
            packed_state: [0; PACKED_WIDTH * GFX_HEIGHT_DEFAULT],
            was_modified: false,
            modification: Bounds::new(0),
        }
    }

    pub fn get_current_mode(&self) -> (usize, usize) {
        // @TODO implement different modes and mode selection
        (GFX_WIDTH_DEFAULT, GFX_HEIGHT_DEFAULT)
    }

    pub fn pop_modifications(&mut self) -> Option<ModificationData> {
        let result = if self.was_modified {
            Some(ModificationData {
                offset: self.modification.min * size_of::<Storage>(),
                data: &self.packed_state[self.modification.min..=self.modification.max],
            })
        } else {
            None
        };

        self.was_modified = false;
        result
    }

    pub fn clear(&mut self, clear_with: bool) {
        let (gfx_width, gfx_height) = self.get_current_mode();
        for x in 0..gfx_width {
            for y in 0..gfx_height {
                self.set(x, y, clear_with);
            }
        }
    }

    pub fn get(&self, x: usize, y: usize) -> bool {
        let (col, nibble) = self.get_bucket(x, y);
        let mask = 1 << nibble;
        self.packed_state[col] & mask != 0
    }

    pub fn set(&mut self, x: usize, y: usize, value: bool) {
        let (col, nibble) = self.get_bucket(x, y);
        let mask = 1 << nibble;
        if value {
            self.packed_state[col] |= mask;
        } else {
            self.packed_state[col] &= !mask;
        }
        self.extend_modification(col);
    }

    fn extend_modification(&mut self, col: usize) {
        if self.was_modified {
            self.modification.extend(col)
        } else {
            self.modification = Bounds::new(col);
            self.was_modified = true
        }
    }

    #[inline]
    fn get_bucket(&self, x: usize, y: usize) -> (usize, usize) {
        let (gfx_width, _) = self.get_current_mode();
        let real_x = x / STORAGE_BITS;
        let col = (y * (gfx_width / STORAGE_BITS)) + real_x;
        let nibble = (STORAGE_BITS * (real_x + 1)) - x - 1;
        (col, nibble)
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use super::DisplayState;
    use proptest::prelude::*;

    #[test]
    fn modification_test() {
        let mut gfx = DisplayState::new();
        gfx.set(0, 0, true);

        let modification = gfx.pop_modifications().expect("No modifications");
        assert_eq!(modification.data.len(), 1);
        assert_eq!(modification.offset, 0);

        gfx.set(63, 31, true);

        let modification = gfx.pop_modifications().expect("No modifications");
        assert_eq!(modification.data.len(), 1);
        assert_eq!(modification.offset, 63 * size_of::<u32>());

        gfx.set(63, 31, true);
        gfx.set(0, 0, true);

        let modification = gfx.pop_modifications().expect("No modifications");
        assert_eq!(modification.data.len(), 64);
        assert_eq!(modification.offset, 0);

        if let Some(_) = gfx.pop_modifications() {
            assert!(false);
        }
    }

    proptest! {
        #[test]
        fn compression_proptest(
            coords in prop::collection::vec(
                (0..super::GFX_WIDTH_DEFAULT, 0..super::GFX_HEIGHT_DEFAULT),
                1..500
            )
        ) {
            let mut gfx = DisplayState::new();
            // Set
            for (x, y) in coords.iter() {
                gfx.set(*x, *y, true);
            }
            for x in 0..super::GFX_WIDTH_DEFAULT {
                for y in 0..super::GFX_HEIGHT_DEFAULT {
                    let res = gfx.get(x, y);
                    assert!(res == coords.contains(&(x, y)));
                }
            }
            // Reset
            for (x, y) in coords.iter() {
                gfx.set(*x, *y, false);
            }
            for x in 0..super::GFX_WIDTH_DEFAULT {
                for y in 0..super::GFX_HEIGHT_DEFAULT {
                    assert!(gfx.get(x, y) == false);
                }
            }
        }
    }
}
