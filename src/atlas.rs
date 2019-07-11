use serde::Serialize;
use std::collections::HashMap;
use std::cmp::Reverse;

#[derive(Debug, Serialize)]
pub struct Atlas {
    pub textures: HashMap<String, Rect>,
    pub size: Size,
}

#[derive(Debug, Serialize, Copy, Clone)]
pub struct Rect {
    pub left: u32,
    pub top: u32,
    #[serde(flatten)]
    pub size: Size,
}

impl Rect {
    /// Insert rectangle of `size` into top left corner of rect. Remaning space
    /// being split again into two rectangles, and smaller one returned
    ///
    /// +--------+           +--------+
    /// |        |           | r | n  |
    /// |        |   +---+   +---+----|
    /// |   c    | + | r | = |        |
    /// |        |   +---+   |   c    |
    /// |        |           |        |
    /// |        |           |        |
    /// +--------+           +--------+
    ///
    fn insert(&mut self, size: Size) -> Option<(Rect, Option<Rect>)> {
        if self.size == size {
            info!("Inserted image has same size as target rect. Results in empty space of size 0");
        }
        if size.fit_in(self.size) {
            let r = Rect {
                left: self.left,
                top: self.top,
                size,
            };
            let other = if size.width == self.size.width {
                self.size.height -= size.height;
                self.top += size.height;
                None
            } else if size.height == self.size.height {
                self.size.width -= size.width;
                self.left += size.width;
                None
            } else {
                // TODO: Experiment with horizontal vs vertical splitting
                let rect = Rect {
                    left: self.left + size.width,
                    top: self.top,
                    size: Size {
                        width: self.size.width - size.width,
                        height: size.height,
                    },
                };
                self.size.height -= size.height;
                self.top += size.height;
                Some(rect)
            };
            Some((r, other))
        } else {
            None
        }
    }

    fn bound_size(&self) -> Size {
        Size {
            width: self.left + self.size.width,
            height: self.top + self.size.height,
        }
    }
}

#[derive(Copy, Clone, Debug, Serialize, Eq, PartialEq)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    fn zero() -> Self {
        Self {
            width: 0,
            height: 0,
        }
    }

    fn fit_in(&self, size: Size) -> bool {
        self.width <= size.width && self.height <= size.height
    }

    fn max(self, other: Size) -> Size {
        Self {
            width: self.width.max(other.width),
            height: self.height.max(other.height),
        }
    }
}

/// Pack non repeating images into texture atlas of fixed size.
pub struct AtlasBuilder {
    empty_spaces: Vec<Rect>,
    textures: HashMap<String, Rect>,
}

impl AtlasBuilder {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            empty_spaces: vec![Rect {
                left: 0,
                top: 0,
                size: Size { width, height },
            }],
            textures: HashMap::new(),
        }
    }

    pub fn get_map(self) -> Atlas{
        let size = self.min_bounding_rect();
        Atlas{
            textures: self.textures,
            size
        }
    }

    pub fn build<T>(&mut self, images: T) -> Option<()>
    where
        T: IntoIterator<Item = (String, Size)>,
    {
        let mut data: Vec<_> = images.into_iter().collect();
        data.sort_by_key(|(_name, size)| Reverse(size.height * size.width));
        for (name, size) in data {
            self.add_rect(name, size)?;
        }
        Some(())
    }

    fn add_rect(&mut self, name: String, mut size: Size) -> Option<()> {
        for space in self.empty_spaces.iter_mut().rev() {
            if let Some((texture_rect, new_empty)) = space.insert(size) {
                if let Some(_old) = self.textures.insert(name.clone(), texture_rect){
                    warn!("Image {:?} inserted multiple times", &name);
                }
                if let Some(space) = new_empty {
                    self.empty_spaces.push(space);
                }
                return Some(());
            }
        }
        None
    }

    pub fn min_bounding_rect(&self) -> Size {
        self.textures
            .values()
            .map(Rect::bound_size)
            .fold(Size::zero(), Size::max)
    }
}
