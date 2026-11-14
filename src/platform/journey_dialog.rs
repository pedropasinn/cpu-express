//! Native path selection only: validation and journey I/O belong to the caller.

use std::path::PathBuf;

/// Select a JSON journey path on the UI thread (`save=true` exports).
///
/// Returns `Ok(None)` when the user cancels. This synchronous, modal call must
/// be triggered once by a button, never from the render loop unconditionally.
/// It neither reads nor writes the selected file. The caller must validate an
/// import before replacing state and handle export I/O errors independently.
/// Windows requires the windows-sys feature `Win32_UI_Controls_Dialogs`.
/// A save selection may replace an existing file only after the native prompt;
/// the caller must use the returned path unchanged to honor that confirmation.
pub fn choose_journey_file(save: bool) -> Result<Option<PathBuf>, String> {
    #[cfg(windows)]
    {
        native::choose(save)
    }
    #[cfg(not(windows))]
    {
        let _ = save;
        Err("O seletor de arquivos de jornada requer Windows.".to_owned())
    }
}

#[cfg(windows)]
mod native {
    use std::{ffi::OsString, mem, os::windows::ffi::OsStringExt, path::PathBuf, ptr};
    use windows_sys::Win32::{
        System::Threading::{GetCurrentProcessId, GetCurrentThreadId},
        UI::{
            Controls::Dialogs::{
                CommDlgExtendedError, GetOpenFileNameW, GetSaveFileNameW, OFN_EXPLORER,
                OFN_FILEMUSTEXIST, OFN_NOCHANGEDIR, OFN_NOTESTFILECREATE, OFN_OVERWRITEPROMPT,
                OFN_PATHMUSTEXIST, OPENFILENAMEW,
            },
            WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId},
        },
    };

    pub(super) fn choose(save: bool) -> Result<Option<PathBuf>, String> {
        let filter: Vec<u16> = "Jornada CPU Express (*.json)\0*.json\0\0"
            .encode_utf16()
            .collect();
        let extension: Vec<u16> = "json\0".encode_utf16().collect();
        let title: Vec<u16> = if save {
            "Exportar jornada\0"
        } else {
            "Importar jornada\0"
        }
        .encode_utf16()
        .collect();
        // Single selection with space for a long Windows path. No MAX_PATH truncation.
        let mut path = vec![0u16; 32_768];
        if save {
            let name: Vec<u16> = "jornada.json".encode_utf16().collect();
            path[..name.len()].copy_from_slice(&name);
        }
        // SAFETY: read-only queries. Only a foreground window of this process
        // AND this UI thread can own the dialog; otherwise use no owner.
        let owner = unsafe {
            let candidate = GetForegroundWindow();
            let mut process = 0;
            let thread = GetWindowThreadProcessId(candidate, &mut process);
            if !candidate.is_null()
                && process == GetCurrentProcessId()
                && thread == GetCurrentThreadId()
            {
                candidate
            } else {
                ptr::null_mut()
            }
        };
        let mut dialog = OPENFILENAMEW {
            lStructSize: mem::size_of::<OPENFILENAMEW>() as u32,
            hwndOwner: owner,
            lpstrFilter: filter.as_ptr(),
            nFilterIndex: 1,
            lpstrFile: path.as_mut_ptr(),
            nMaxFile: path.len() as u32,
            lpstrTitle: title.as_ptr(),
            lpstrDefExt: extension.as_ptr(),
            Flags: OFN_EXPLORER
                | OFN_NOCHANGEDIR
                | OFN_PATHMUSTEXIST
                | if save {
                    OFN_OVERWRITEPROMPT | OFN_NOTESTFILECREATE
                } else {
                    OFN_FILEMUSTEXIST
                },
            ..Default::default()
        };
        // SAFETY: correctly sized initialized structure; all UTF-16 buffers
        // remain alive throughout the synchronous call, with writable file
        // storage of nMaxFile units. No hooks or borrowed callbacks are installed.
        let selected = unsafe {
            if save {
                GetSaveFileNameW(&mut dialog)
            } else {
                GetOpenFileNameW(&mut dialog)
            }
        };
        if selected == 0 {
            // SAFETY: query immediately after the common dialog returns failure.
            let error = unsafe { CommDlgExtendedError() };
            return if error == 0 {
                Ok(None)
            } else {
                Err(format!(
                    "Falha no seletor de jornada (Common Dialog 0x{error:08X})."
                ))
            };
        }
        decode_path(&path).map(Some)
    }

    fn decode_path(buffer: &[u16]) -> Result<PathBuf, String> {
        let length = buffer
            .iter()
            .position(|unit| *unit == 0)
            .ok_or_else(|| "O seletor retornou um caminho sem terminador.".to_owned())?;
        if length == 0 {
            return Err("O seletor retornou um caminho vazio.".to_owned());
        }
        // Preserve Windows paths losslessly, including unpaired UTF-16 surrogates.
        Ok(PathBuf::from(OsString::from_wide(&buffer[..length])))
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::os::windows::ffi::OsStrExt;

        #[test]
        fn selected_path_is_lossless_and_stops_at_terminator() {
            let units = [67, 58, 92, 0xD800, 46, 106, 115, 111, 110, 0, 99];
            let path = decode_path(&units).unwrap();
            assert_eq!(
                path.as_os_str().encode_wide().collect::<Vec<_>>(),
                units[..9]
            );
        }

        #[test]
        fn malformed_dialog_results_are_not_paths() {
            assert!(decode_path(&[]).is_err());
            assert!(decode_path(&[0]).is_err());
            assert!(decode_path(&[65, 66]).is_err());
        }
    }
}
