//! Iced implementation of AutoUI's backend-neutral `ImageSurface` widget.
//!
//! This module is intentionally a Task 1 seam.  Layout, drawing, resource
//! handling, and pointer events are introduced by later Plan 547 tasks.

use iced::{Color, Point, Rectangle, Size};

/// User-facing fit policies accepted by ImageSurface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageSurfaceFit {
    /// Preserve aspect ratio and fit the whole image in the viewport.
    Contain,
    /// Preserve aspect ratio and fit the image width to the viewport.
    Width,
    /// Draw source pixels one-for-one before applying the interactive zoom.
    OneToOne,
    /// Do not apply a viewport fit; use the source dimensions directly.
    Free,
}

impl ImageSurfaceFit {
    /// Parse the stable names used by Auto/Vue/Rust generators.
    pub fn parse(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "width" | "fit-width" | "fit_width" => Self::Width,
            "1:1" | "one-to-one" | "one_to_one" | "actual" => Self::OneToOne,
            "free" | "none" => Self::Free,
            _ => Self::Contain,
        }
    }
}

/// Result of laying out an ImageSurface for one viewport.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ImageSurfaceGeometry {
    /// The clipping rectangle in the parent's coordinate space.
    pub clip: Rectangle,
    /// The unrotated, scaled image rectangle in the parent's coordinate space.
    pub image: Rectangle,
    /// Axis-aligned bounds after applying rotation, useful for hit testing.
    pub rotated_bounds: Rectangle,
    /// Effective scale after fit and interactive zoom.
    pub scale: f32,
    /// Normalized rotation in degrees, the original value modulo 360.
    pub rotation: f32,
}

impl ImageSurfaceGeometry {
    /// Returns whether a point is inside the clipped viewport.
    pub fn contains(&self, point: Point) -> bool {
        self.clip.contains(point)
    }
}

/// Compute contain/width/1:1/free geometry, including translation and
/// rotation. Zero-sized source images intentionally produce a zero image
/// rectangle while retaining the viewport clip for placeholder/error paint.
pub fn image_surface_geometry(
    viewport: Rectangle,
    source: Size,
    fit: ImageSurfaceFit,
    zoom: f32,
    offset_x: f32,
    offset_y: f32,
    rotation_degrees: f32,
) -> ImageSurfaceGeometry {
    let source_width = source.width.max(0.0);
    let source_height = source.height.max(0.0);
    let fit_scale = if source_width <= f32::EPSILON || source_height <= f32::EPSILON {
        0.0
    } else {
        match fit {
            ImageSurfaceFit::Contain => {
                (viewport.width / source_width).min(viewport.height / source_height)
            }
            ImageSurfaceFit::Width => viewport.width / source_width,
            ImageSurfaceFit::OneToOne | ImageSurfaceFit::Free => 1.0,
        }
    };
    let zoom = zoom.clamp(0.05, 64.0);
    let scale = (fit_scale * zoom).max(0.0);
    let image_size = Size::new(source_width * scale, source_height * scale);
    let center = Point::new(
        viewport.x + viewport.width / 2.0 + offset_x,
        viewport.y + viewport.height / 2.0 + offset_y,
    );
    let image = Rectangle {
        x: center.x - image_size.width / 2.0,
        y: center.y - image_size.height / 2.0,
        width: image_size.width,
        height: image_size.height,
    };

    let rotation = rotation_degrees.rem_euclid(360.0);
    let radians = rotation.to_radians();
    let (sin, cos) = radians.sin_cos();
    let rotated_width = image.width * cos.abs() + image.height * sin.abs();
    let rotated_height = image.width * sin.abs() + image.height * cos.abs();
    let rotated_bounds = Rectangle {
        x: center.x - rotated_width / 2.0,
        y: center.y - rotated_height / 2.0,
        width: rotated_width,
        height: rotated_height,
    };

    ImageSurfaceGeometry {
        clip: viewport,
        image,
        rotated_bounds,
        scale,
        rotation,
    }
}

/// Alias used by renderers that describe the operation as a layout pass.
pub fn layout_geometry(
    viewport: Rectangle,
    source: Size,
    fit: ImageSurfaceFit,
    zoom: f32,
    offset_x: f32,
    offset_y: f32,
    rotation_degrees: f32,
) -> ImageSurfaceGeometry {
    image_surface_geometry(
        viewport,
        source,
        fit,
        zoom,
        offset_x,
        offset_y,
        rotation_degrees,
    )
}

/// Resource state consumed by the paint path. It deliberately carries no
/// bytes: decoded pixels remain in the bounded media pipeline/cache.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImageSurfaceResource {
    Loading,
    Ready,
    Error(String),
    Placeholder,
}

/// Sampling quality requested by an ImageSurface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageSurfaceFilter {
    Linear,
    High,
}

/// Backend-neutral paint decision for the Iced widget.
#[derive(Debug, Clone, PartialEq)]
pub struct ImageSurfacePaint {
    pub background: Color,
    pub resource: ImageSurfaceResource,
    pub filter: ImageSurfaceFilter,
}

/// Select stable loading/error/placeholder visuals without touching media
/// bytes. A quality of 0–79 requests linear sampling; high quality keeps the
/// sharper/high filter used for settled renditions.
pub fn image_surface_paint(
    quality: u8,
    resource: ImageSurfaceResource,
) -> ImageSurfacePaint {
    ImageSurfacePaint {
        background: Color::from_rgba(0.08, 0.09, 0.12, 1.0),
        resource,
        filter: if quality < 80 {
            ImageSurfaceFilter::Linear
        } else {
            ImageSurfaceFilter::High
        },
    }
}

/// Pan lifecycle reported to the backend-neutral callback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageSurfacePanPhase {
    Start,
    Move,
    End,
}

/// Raw pointer/media events received by the Iced surface.
#[derive(Debug, Clone, PartialEq)]
pub enum ImageSurfaceInputEvent {
    Wheel { delta_y: f32, cursor: Point },
    PanStart { cursor: Point },
    PanMove { cursor: Point },
    PanEnd,
    DoubleClick { cursor: Point },
    Loaded { width: u32, height: u32 },
    Error { code: String, message: String },
}

/// Normalized payload delivered to ImageSurface callbacks.
#[derive(Debug, Clone, PartialEq)]
pub enum ImageSurfaceEvent {
    Wheel { delta_y: f32, x: f32, y: f32 },
    Pan { dx: f32, dy: f32, phase: ImageSurfacePanPhase },
    DoubleClick { x: f32, y: f32 },
    Loaded { width: u32, height: u32, revision: u64 },
    Error { code: String, message: String, revision: u64 },
}

/// Per-source input state. Changing the source revision resets pan and the
/// one-shot load/error gates, so stale callbacks can never leak across images.
#[derive(Debug, Clone, PartialEq)]
pub struct ImageSurfaceInputState {
    pub src_revision: u64,
    pan_last: Option<Point>,
    pub load_emitted: bool,
    pub error_emitted: bool,
}

impl ImageSurfaceInputState {
    pub fn new(src_revision: u64) -> Self {
        Self {
            src_revision,
            pan_last: None,
            load_emitted: false,
            error_emitted: false,
        }
    }

    pub fn set_source_revision(&mut self, src_revision: u64) {
        if self.src_revision != src_revision {
            self.src_revision = src_revision;
            self.pan_last = None;
            self.load_emitted = false;
            self.error_emitted = false;
        }
    }
}

fn surface_local(viewport: Rectangle, cursor: Point) -> Point {
    Point::new(cursor.x - viewport.x, cursor.y - viewport.y)
}

/// Normalize one Iced event. revision is the source revision associated with
/// the event; mismatches are dropped before any state or callback mutation.
pub fn normalize_image_surface_event(
    state: &mut ImageSurfaceInputState,
    viewport: Rectangle,
    revision: u64,
    event: ImageSurfaceInputEvent,
) -> Option<ImageSurfaceEvent> {
    if revision != state.src_revision {
        return None;
    }
    match event {
        ImageSurfaceInputEvent::Wheel { delta_y, cursor } => {
            let local = surface_local(viewport, cursor);
            Some(ImageSurfaceEvent::Wheel {
                delta_y,
                x: local.x,
                y: local.y,
            })
        }
        ImageSurfaceInputEvent::PanStart { cursor } => {
            let local = surface_local(viewport, cursor);
            state.pan_last = Some(local);
            Some(ImageSurfaceEvent::Pan {
                dx: 0.0,
                dy: 0.0,
                phase: ImageSurfacePanPhase::Start,
            })
        }
        ImageSurfaceInputEvent::PanMove { cursor } => {
            let local = surface_local(viewport, cursor);
            let previous = state.pan_last.replace(local)?;
            Some(ImageSurfaceEvent::Pan {
                dx: local.x - previous.x,
                dy: local.y - previous.y,
                phase: ImageSurfacePanPhase::Move,
            })
        }
        ImageSurfaceInputEvent::PanEnd => {
            if state.pan_last.take().is_none() {
                return None;
            }
            Some(ImageSurfaceEvent::Pan {
                dx: 0.0,
                dy: 0.0,
                phase: ImageSurfacePanPhase::End,
            })
        }
        ImageSurfaceInputEvent::DoubleClick { cursor } => {
            let local = surface_local(viewport, cursor);
            Some(ImageSurfaceEvent::DoubleClick {
                x: local.x,
                y: local.y,
            })
        }
        ImageSurfaceInputEvent::Loaded { width, height } => {
            if state.load_emitted {
                return None;
            }
            state.load_emitted = true;
            Some(ImageSurfaceEvent::Loaded {
                width,
                height,
                revision,
            })
        }
        ImageSurfaceInputEvent::Error { code, message } => {
            if state.error_emitted {
                return None;
            }
            state.error_emitted = true;
            Some(ImageSurfaceEvent::Error {
                code,
                message,
                revision,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_surface_geometry_cases() {
        let viewport = Rectangle {
            x: 10.0,
            y: 20.0,
            width: 800.0,
            height: 600.0,
        };
        let source = Size::new(1600.0, 900.0);
        let contain = image_surface_geometry(
            viewport,
            source,
            ImageSurfaceFit::Contain,
            1.0,
            0.0,
            0.0,
            0.0,
        );
        assert!((contain.scale - 0.5).abs() < f32::EPSILON);
        assert!((contain.image.width - 800.0).abs() < f32::EPSILON);
        assert!((contain.image.height - 450.0).abs() < f32::EPSILON);
        assert!((contain.image.y - 95.0).abs() < f32::EPSILON);

        let width = image_surface_geometry(
            viewport,
            source,
            ImageSurfaceFit::Width,
            2.0,
            12.0,
            -8.0,
            90.0,
        );
        assert!((width.scale - 1.0).abs() < f32::EPSILON);
        assert!((width.image.x - (-378.0)).abs() < f32::EPSILON);
        assert!((width.image.y - (-138.0)).abs() < f32::EPSILON);
        assert!((width.rotated_bounds.width - 900.0).abs() < 0.01);
        assert!(width.contains(Point::new(10.0, 20.0)));

        let one_to_one = image_surface_geometry(
            viewport,
            Size::new(100.0, 50.0),
            ImageSurfaceFit::OneToOne,
            100.0,
            0.0,
            0.0,
            450.0,
        );
        assert!((one_to_one.scale - 64.0).abs() < f32::EPSILON);
        assert!((one_to_one.rotation - 90.0).abs() < f32::EPSILON);
    }

    #[test]
    fn image_surface_paint_states_and_filter() {
        assert_eq!(
            image_surface_paint(70, ImageSurfaceResource::Loading).filter,
            ImageSurfaceFilter::Linear
        );
        assert_eq!(
            image_surface_paint(90, ImageSurfaceResource::Ready).resource,
            ImageSurfaceResource::Ready
        );
        assert_eq!(
            image_surface_paint(90, ImageSurfaceResource::Error("bad image".into())).resource,
            ImageSurfaceResource::Error("bad image".into())
        );
    }

    #[test]
    fn image_surface_events_cases() {
        let viewport = Rectangle {
            x: 10.0,
            y: 20.0,
            width: 800.0,
            height: 600.0,
        };
        let mut state = ImageSurfaceInputState::new(7);
        let wheel = normalize_image_surface_event(
            &mut state,
            viewport,
            7,
            ImageSurfaceInputEvent::Wheel {
                delta_y: -12.0,
                cursor: Point::new(110.0, 70.0),
            },
        );
        assert_eq!(
            wheel,
            Some(ImageSurfaceEvent::Wheel {
                delta_y: -12.0,
                x: 100.0,
                y: 50.0
            })
        );

        assert!(matches!(
            normalize_image_surface_event(
                &mut state,
                viewport,
                7,
                ImageSurfaceInputEvent::PanStart {
                    cursor: Point::new(40.0, 60.0)
                }
            ),
            Some(ImageSurfaceEvent::Pan {
                phase: ImageSurfacePanPhase::Start,
                ..
            })
        ));
        assert_eq!(
            normalize_image_surface_event(
                &mut state,
                viewport,
                7,
                ImageSurfaceInputEvent::PanMove {
                    cursor: Point::new(55.0, 90.0)
                }
            ),
            Some(ImageSurfaceEvent::Pan {
                dx: 15.0,
                dy: 30.0,
                phase: ImageSurfacePanPhase::Move
            })
        );
        assert!(matches!(
            normalize_image_surface_event(
                &mut state,
                viewport,
                7,
                ImageSurfaceInputEvent::PanEnd
            ),
            Some(ImageSurfaceEvent::Pan {
                phase: ImageSurfacePanPhase::End,
                ..
            })
        ));

        assert_eq!(
            normalize_image_surface_event(
                &mut state,
                viewport,
                7,
                ImageSurfaceInputEvent::Loaded {
                    width: 1600,
                    height: 900
                }
            ),
            Some(ImageSurfaceEvent::Loaded {
                width: 1600,
                height: 900,
                revision: 7
            })
        );
        assert!(normalize_image_surface_event(
            &mut state,
            viewport,
            7,
            ImageSurfaceInputEvent::Loaded {
                width: 1600,
                height: 900
            }
        )
        .is_none());
        assert!(normalize_image_surface_event(
            &mut state,
            viewport,
            6,
            ImageSurfaceInputEvent::DoubleClick {
                cursor: Point::new(10.0, 20.0)
            }
        )
        .is_none());

        state.set_source_revision(8);
        assert!(!state.load_emitted);
        assert!(normalize_image_surface_event(
            &mut state,
            viewport,
            8,
            ImageSurfaceInputEvent::Error {
                code: "decode".into(),
                message: "bad image".into()
            }
        )
        .is_some());
        assert!(normalize_image_surface_event(
            &mut state,
            viewport,
            8,
            ImageSurfaceInputEvent::Error {
                code: "decode".into(),
                message: "bad image".into()
            }
        )
        .is_none());
    }
}
