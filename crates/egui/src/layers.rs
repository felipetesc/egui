//! Handles paint layers, i.e. how things
//! are sometimes painted behind or in front of other things.

use crate::{Id, *};
use epaint::{ClippedShape, Shape};

/// Different layer categories
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum Order {
    /// Painted behind all floating windows
    Background,

    /// Special layer between panels and windows
    PanelResizeLine,

    /// Normal moveable windows that you reorder by click
    Middle,

    /// Popups, menus etc that should always be painted on top of windows
    /// Foreground objects can also have tooltips
    Foreground,

    /// Things floating on top of everything else, like tooltips.
    /// You cannot interact with these.
    Tooltip,

    /// Debug layer, always painted last / on top
    Debug,
}

impl Order {
    const COUNT: usize = 6;
    const ALL: [Order; Self::COUNT] = [
        Self::Background,
        Self::PanelResizeLine,
        Self::Middle,
        Self::Foreground,
        Self::Tooltip,
        Self::Debug,
    ];

    #[inline(always)]
    pub fn allow_interaction(&self) -> bool {
        match self {
            Self::Background
            | Self::PanelResizeLine
            | Self::Middle
            | Self::Foreground
            | Self::Debug => true,
            Self::Tooltip => false,
        }
    }

    /// Short and readable summary
    pub fn short_debug_format(&self) -> &'static str {
        match self {
            Self::Background => "backg",
            Self::PanelResizeLine => "panel",
            Self::Middle => "middl",
            Self::Foreground => "foreg",
            Self::Tooltip => "toolt",
            Self::Debug => "debug",
        }
    }
}

/// An identifier for a paint layer.
/// Also acts as an identifier for [`Area`]:s.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct AreaLayerId {
    pub order: Order,
    pub id: Id,
}

/// For backwards-compatibility with `AreaLayerId`
#[deprecated(note = "Use `AreaLayerId` instead")]
pub type LayerId = AreaLayerId;

impl AreaLayerId {
    pub fn new(order: Order, id: Id) -> Self {
        Self { order, id }
    }

    pub fn debug() -> Self {
        Self {
            order: Order::Debug,
            id: Id::new("debug"),
        }
    }

    pub fn background() -> Self {
        Self {
            order: Order::Background,
            id: Id::background(),
        }
    }

    #[inline(always)]
    pub fn allow_interaction(&self) -> bool {
        self.order.allow_interaction()
    }

    #[must_use]
    pub fn with_z(self, z: ZOrder) -> ZLayer {
        ZLayer::from_area_layer_z(self, z)
    }

    /// Short and readable summary
    pub fn short_debug_format(&self) -> String {
        format!(
            "{} {}",
            self.order.short_debug_format(),
            self.id.short_debug_format()
        )
    }
}

/// Represents the relative order an element should be displayed
/// Lower values render first, and therefore appear below higher values
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct ZOrder(pub i32);

impl ZOrder {
    const BASE: ZOrder = ZOrder(0);
    const ABOVE_ALL: ZOrder = ZOrder(i32::MAX);
    const BELOW_ALL: ZOrder = ZOrder(i32::MIN);

    /// Base z-order, corresponding to a level of 0
    #[inline(always)]
    pub fn base() -> Self {
        Self::BASE
    }

    /// Maximal z-order - nothing on the same area layer can render above this
    #[inline(always)]
    pub fn above_all() -> Self {
        Self::ABOVE_ALL
    }

    /// Minimal z-order - nothing on the same area layer can render below this
    #[inline(always)]
    pub fn below_all() -> Self {
        Self::BELOW_ALL
    }

    /// Directly above
    pub fn above(self) -> Self {
        self.above_by(1)
    }

    /// Directly below
    pub fn below(self) -> Self {
        self.below_by(1)
    }

    /// Above by the number of levels given
    pub fn above_by(self, levels: i32) -> Self {
        Self(self.0.saturating_add(levels))
    }

    /// Below by the number of levels given
    pub fn below_by(self, levels: i32) -> Self {
        Self(self.0.saturating_sub(levels))
    }
}

impl Default for ZOrder {
    fn default() -> Self {
        Self::BASE
    }
}

/// An identifier for a paint layer which supports Z-indexing
///
/// This says: draw on `area_layer` with index z. This only affects the display
/// order of elements on the same area layer. Order of area layers still takes
/// precedence over z-index.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct ZLayer {
    pub area_layer: AreaLayerId,
    pub z: ZOrder,
}

impl ZLayer {
    pub fn new(order: Order, id: Id, z: ZOrder) -> Self {
        Self {
            area_layer: AreaLayerId { order, id },
            z,
        }
    }

    /// Use specified Z-level
    pub fn from_area_layer_z(area_layer: AreaLayerId, z: ZOrder) -> Self {
        Self { area_layer, z }
    }

    /// Use base Z-level
    pub fn from_area_layer(area_layer: AreaLayerId) -> Self {
        Self::from_area_layer_z(area_layer, ZOrder::default())
    }

    pub fn debug() -> Self {
        Self::from_area_layer(AreaLayerId::debug())
    }

    pub fn background() -> Self {
        Self::from_area_layer(AreaLayerId::background())
    }

    #[must_use]
    pub fn with_z(self, z: ZOrder) -> Self {
        Self::from_area_layer_z(self.area_layer, z)
    }

    /// Get the `ZLayer` directly above this one
    #[must_use]
    pub fn above(self) -> Self {
        self.with_z(self.z.above())
    }

    /// Get the `ZLayer` above this one by `levels` levels
    #[must_use]
    pub fn above_by(self, levels: i32) -> Self {
        self.with_z(self.z.above_by(levels))
    }

    /// Get the `ZLayer` directly below this one
    #[must_use]
    pub fn below(self) -> Self {
        self.with_z(self.z.below())
    }

    /// Get the `ZLayer` below this one by `levels` levels
    #[must_use]
    pub fn below_by(self, levels: i32) -> Self {
        self.with_z(self.z.below_by(levels))
    }

    /// `Id` of underlying area layer
    #[inline(always)]
    pub fn id(&self) -> Id {
        self.area_layer.id
    }

    /// `Order` of underlying area layer
    #[inline(always)]
    pub fn order(&self) -> Order {
        self.area_layer.order
    }
}

/// A unique identifier of a specific [`Shape`] in a [`PaintList`].
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ShapeIdx(usize);

/// A list of [`Shape`]s paired with a clip rectangle.
#[derive(Clone, Default)]
pub struct PaintList(Vec<(ZOrder, ClippedShape)>);

impl PaintList {
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the index of the new [`Shape`] that can be used with `PaintList::set`.
    #[inline(always)]
    pub fn add(&mut self, clip_rect: Rect, shape: Shape, z: ZOrder) -> ShapeIdx {
        let idx = ShapeIdx(self.0.len());
        self.0.push((z, ClippedShape(clip_rect, shape)));
        idx
    }

    pub fn extend<I: IntoIterator<Item = Shape>>(&mut self, clip_rect: Rect, shapes: I, z: ZOrder) {
        self.0.extend(
            shapes
                .into_iter()
                .map(|shape| (z, ClippedShape(clip_rect, shape))),
        );
    }

    /// Modify an existing [`Shape`].
    ///
    /// Sometimes you want to paint a frame behind some contents, but don't know how large the frame needs to be
    /// until the contents have been added, and therefor also painted to the [`PaintList`].
    ///
    /// The solution is to allocate a [`Shape`] using `let idx = paint_list.add(cr, Shape::Noop);`
    /// and then later setting it using `paint_list.set(idx, cr, frame);`.
    #[inline(always)]
    pub fn set(&mut self, idx: ShapeIdx, clip_rect: Rect, shape: Shape) {
        self.0[idx.0].1 = ClippedShape(clip_rect, shape);
    }

    /// Translate each [`Shape`] and clip rectangle by this much, in-place
    pub fn translate(&mut self, delta: Vec2) {
        for (.., ClippedShape(clip_rect, shape)) in &mut self.0 {
            *clip_rect = clip_rect.translate(delta);
            shape.translate(delta);
        }
    }
}

#[derive(Clone, Default)]
pub(crate) struct GraphicLayers([IdMap<PaintList>; Order::COUNT]);

impl GraphicLayers {
    pub fn list(&mut self, layer_id: AreaLayerId) -> &mut PaintList {
        self.0[layer_id.order as usize]
            .entry(layer_id.id)
            .or_default()
    }

    pub fn drain(
        &mut self,
        area_order: &[AreaLayerId],
    ) -> impl ExactSizeIterator<Item = ClippedShape> {
        let mut all_shapes: Vec<_> = Default::default();

        for &order in &Order::ALL {
            let order_map = &mut self.0[order as usize];

            // Sort by z-order
            for list in order_map.values_mut() {
                list.0.sort_by_key(|(z, ..)| *z);
            }

            // If a layer is empty at the start of the frame
            // then nobody has added to it, and it is old and defunct.
            // Free it to save memory:
            order_map.retain(|_, list| !list.is_empty());

            // First do the layers part of area_order:
            for layer_id in area_order {
                if layer_id.order == order {
                    if let Some(list) = order_map.get_mut(&layer_id.id) {
                        all_shapes.append(&mut list.0);
                    }
                }
            }

            // Also draw areas that are missing in `area_order`:
            for shapes in order_map.values_mut() {
                all_shapes.append(&mut shapes.0);
            }
        }

        all_shapes.into_iter().map(|(.., shape)| shape)
    }
}
