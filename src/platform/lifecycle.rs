//! Pure policy for desktop visibility. A transition is only committed by the
//! Windows adapter after the corresponding native operation has succeeded.

use std::fmt;

/// Last observed visibility, or terminal shutdown of the controller.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DesktopState {
    #[default]
    Visible,
    Minimized,
    HiddenByUser,
    Closed,
}

/// Explicit user actions. Hiding never means closing and closing never hides.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DesktopAction {
    Show,
    Minimize,
    Hide,
    Close,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DesktopTransitionError {
    AlreadyClosed,
    NoRecoveryTray,
}

impl fmt::Display for DesktopTransitionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::AlreadyClosed => "O controlador de desktop já foi encerrado.",
            Self::NoRecoveryTray => {
                "Não é possível ocultar: a bandeja não está disponível para restaurar a janela."
            }
        })
    }
}

impl std::error::Error for DesktopTransitionError {}

impl DesktopState {
    /// Suspend animation after a real hide/minimize, but allow eframe's first
    /// paint while its anti-flash startup window is still invisible. Treating
    /// that initial invisibility as a user pause prevents eframe showing it.
    #[must_use]
    pub fn suspends_animation(self, has_been_shown: bool) -> bool {
        match self {
            Self::Visible => false,
            Self::HiddenByUser => has_been_shown,
            Self::Minimized | Self::Closed => true,
        }
    }

    /// Check a proposed action without changing any state or touching an OS.
    /// `tray_available` means a successfully registered tray at the last check,
    /// not a guarantee that Explorer can never exit between two operations.
    pub fn after(
        self,
        action: DesktopAction,
        tray_available: bool,
    ) -> Result<Self, DesktopTransitionError> {
        if action == DesktopAction::Close {
            return Ok(Self::Closed);
        }
        if self == Self::Closed {
            return Err(DesktopTransitionError::AlreadyClosed);
        }
        match action {
            DesktopAction::Show => Ok(Self::Visible),
            DesktopAction::Minimize => Ok(Self::Minimized),
            DesktopAction::Hide if tray_available => Ok(Self::HiddenByUser),
            DesktopAction::Hide => Err(DesktopTransitionError::NoRecoveryTray),
            DesktopAction::Close => Ok(Self::Closed),
        }
    }

    /// Loss of the recovery path must make a hidden window visible again.
    /// This is a recovery request; the adapter still has to check native success.
    #[must_use]
    pub fn needs_recovery(self, tray_available: bool) -> bool {
        self == Self::HiddenByUser && !tray_available
    }
}
