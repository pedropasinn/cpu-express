//! Manipulação da faixa em pixels físicos. A configuração só recebe o resultado ao soltar.
use super::{PhysicalRect, WindowPlacement, taskbar_placement};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StripHandle {
    Move,
    Left,
    Right,
    Top,
}

#[derive(Clone, Copy, Debug)]
pub struct StripGesture {
    pub initial: WindowPlacement,
    pub current: WindowPlacement,
    pub area: PhysicalRect,
    pub handle: StripHandle,
    start: [i32; 2],
    pub moved: bool,
}

impl StripGesture {
    pub fn new(
        initial: WindowPlacement,
        area: PhysicalRect,
        handle: StripHandle,
        start: [i32; 2],
    ) -> Self {
        Self {
            initial,
            current: initial,
            area,
            handle,
            start,
            moved: false,
        }
    }

    pub fn update(&mut self, pointer: [i32; 2], fit_height: bool) {
        let delta = [
            pointer[0].saturating_sub(self.start[0]),
            pointer[1].saturating_sub(self.start[1]),
        ];
        self.moved |= delta[0].unsigned_abs().max(delta[1].unsigned_abs()) >= 4;
        if self.moved {
            self.current = resize_strip(self.area, self.initial, self.handle, delta, fit_height)
                .unwrap_or(self.initial);
        }
    }
}

/// Altura livre cresce para cima, mantendo a base da barra; pode cobrir a área de trabalho.
pub fn strip_placement(
    area: PhysicalRect,
    offset: i32,
    width: i32,
    height: i32,
    fit_height: bool,
) -> Option<WindowPlacement> {
    let mut p = taskbar_placement(area, offset, width, 0)?;
    p.width = p.width.max(160.min(area.right.checked_sub(area.left)?));
    p.x = p.x.clamp(area.left, area.right.checked_sub(p.width)?);
    if !fit_height {
        p.height = if height == 0 {
            p.height
        } else {
            height.clamp(24, 240)
        };
        p.y = area.bottom.checked_sub(p.height)?;
    }
    Some(p)
}

pub fn resize_strip(
    area: PhysicalRect,
    initial: WindowPlacement,
    handle: StripHandle,
    delta: [i32; 2],
    fit_height: bool,
) -> Option<WindowPlacement> {
    let p = strip_placement(
        area,
        initial.x.checked_sub(area.left)?,
        initial.width,
        initial.height,
        fit_height,
    )?;
    let right = p.x.checked_add(p.width)?;
    let minimum = 160.min(area.right.checked_sub(area.left)?);
    let mut result = p;
    match handle {
        StripHandle::Move => {
            result.x =
                p.x.saturating_add(delta[0])
                    .clamp(area.left, area.right - p.width)
        }
        StripHandle::Left => {
            result.x =
                p.x.saturating_add(delta[0])
                    .clamp(area.left, right - minimum);
            result.width = right - result.x;
        }
        StripHandle::Right => {
            result.width = p
                .width
                .saturating_add(delta[0])
                .clamp(minimum, area.right - p.x)
        }
        StripHandle::Top if !fit_height => {
            result.height = p.height.saturating_sub(delta[1]).clamp(24, 240);
            result.y = area.bottom.checked_sub(result.height)?;
        }
        StripHandle::Top => (),
    }
    Some(result)
}

/// Só comprimentos locais são convertidos por DPI; a origem virtual permanece física.
pub fn strip_handle(
    local: [f32; 2],
    size: [f32; 2],
    pixels_per_point: f32,
    fit_height: bool,
) -> StripHandle {
    let scale = if pixels_per_point.is_finite() {
        pixels_per_point.max(0.1)
    } else {
        1.0
    };
    if local[0] * scale <= 6.0 {
        StripHandle::Left
    } else if (size[0] - local[0]) * scale <= 6.0 {
        StripHandle::Right
    } else if !fit_height && local[1] * scale <= 6.0 {
        StripHandle::Top
    } else {
        StripHandle::Move
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const BAR: PhysicalRect = PhysicalRect {
        left: -1920,
        top: 1020,
        right: 0,
        bottom: 1080,
    };
    #[test]
    fn limites_e_borda_esquerda_preservam_direita() {
        let p = strip_placement(BAR, 200, 400, 0, true).unwrap();
        let left = resize_strip(BAR, p, StripHandle::Left, [900, 0], true).unwrap();
        assert_eq!(left.width, 160);
        assert_eq!(left.x + left.width, p.x + p.width);
        assert_eq!(
            resize_strip(BAR, p, StripHandle::Move, [i32::MAX, 0], true)
                .unwrap()
                .x,
            -400
        );
        assert_eq!(
            resize_strip(BAR, p, StripHandle::Right, [i32::MAX, 0], true)
                .unwrap()
                .width,
            1720
        );
        assert_eq!(
            strip_placement(BAR, 32768, 32768, 0, true).unwrap().width,
            1920
        );
    }
    #[test]
    fn altura_livre_e_dpi() {
        let p = strip_placement(BAR, 0, 400, 240, false).unwrap();
        assert_eq!((p.y, p.height), (840, 240));
        assert_eq!(strip_placement(BAR, 0, 400, 240, true).unwrap().height, 60);
        for scale in [1.0, 1.25, 1.5, 2.0] {
            assert_eq!(
                strip_handle(
                    [5.0 / scale, 20.0],
                    [400.0 / scale, 60.0 / scale],
                    scale,
                    true
                ),
                StripHandle::Left
            );
            assert_eq!(
                strip_handle(
                    [200.0 / scale, 5.0 / scale],
                    [400.0 / scale, 60.0 / scale],
                    scale,
                    false
                ),
                StripHandle::Top
            );
        }
    }

    #[test]
    fn barra_menor_que_minimo_usa_toda_a_largura_sem_panico() {
        let area = PhysicalRect {
            left: 0,
            top: 0,
            right: 120,
            bottom: 24,
        };
        for width in [0, 160, 32768] {
            let p = strip_placement(area, 900, width, 0, true).unwrap();
            assert_eq!((p.x, p.width), (0, 120));
            for handle in [StripHandle::Move, StripHandle::Left, StripHandle::Right] {
                assert_eq!(resize_strip(area, p, handle, [500, 0], true), Some(p));
            }
        }
    }
    #[test]
    fn clique_e_arraste_usam_limiar_fisico() {
        let p = strip_placement(BAR, 0, 400, 0, true).unwrap();
        let mut g = StripGesture::new(p, BAR, StripHandle::Move, [100, 1050]);
        g.update([103, 1050], true);
        assert!(!g.moved);
        assert_eq!(g.current, p);
        g.update([104, 1050], true);
        assert!(g.moved);
        assert_eq!(g.current.x, p.x + 4);
    }
}
