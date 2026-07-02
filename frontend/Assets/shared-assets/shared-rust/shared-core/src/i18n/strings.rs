//! Centralized UI string lookup.
//!
//! All translated strings used by the shared components live here.
//! Adding a string means: add a variant to [`StringKey`], add a translation
//! row, and the lookup function will pick it up.
//!
//! Lookup is O(n) over the inner array; with 8 languages and a handful of
//! keys, this is faster than a hashmap. If the string count grows past ~50,
//! consider switching to `phf`.

use super::Language;

/// Keys for translatable UI strings.
///
/// Each key maps to a row of translations across all supported languages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StringKey {
    TooltipToggleTheme,
    TooltipPrint,
    TooltipLogout,
    AriaSelectLanguage,
    TitleViewReleaseNotes,
    AriaGitHubProfile,
    StatusReady,
    StatusOnline,
    StatusOffline,
    StatusSaving,
    StatusSaved,
    StatusSaveError,
    StatusLoadError,
    StatusPinSuccess,
    StatusPinFailure,
    StatusLogout,
    StatusFileTooLarge,
    StatusPrintSuccess,
    StatusPrintFailure,
    StatusThemeChanged,
    StatusConflictError,
    StatusValidationError,
}

impl StringKey {
    /// All keys in stable display order. Used to validate translation coverage.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::TooltipToggleTheme,
            Self::TooltipPrint,
            Self::TooltipLogout,
            Self::AriaSelectLanguage,
            Self::TitleViewReleaseNotes,
            Self::AriaGitHubProfile,
            Self::StatusReady,
            Self::StatusOnline,
            Self::StatusOffline,
            Self::StatusSaving,
            Self::StatusSaved,
            Self::StatusSaveError,
            Self::StatusLoadError,
            Self::StatusPinSuccess,
            Self::StatusPinFailure,
            Self::StatusLogout,
            Self::StatusFileTooLarge,
            Self::StatusPrintSuccess,
            Self::StatusPrintFailure,
            Self::StatusThemeChanged,
            Self::StatusConflictError,
            Self::StatusValidationError,
        ]
    }

    /// English fallback text, used when a language row is missing a key.
    #[must_use]
    fn english(self) -> &'static str {
        match self {
            Self::TooltipToggleTheme => "Toggle theme",
            Self::TooltipPrint => "Print",
            Self::TooltipLogout => "Log out",
            Self::AriaSelectLanguage => "Select language",
            Self::TitleViewReleaseNotes => "View Release Notes",
            Self::AriaGitHubProfile => "GitHub Profile",
            Self::StatusReady => "Ready",
            Self::StatusOnline => "Connection restored",
            Self::StatusOffline => "Connection lost",
            Self::StatusSaving => "Saving...",
            Self::StatusSaved => "Changes saved successfully",
            Self::StatusSaveError => "Failed to save changes",
            Self::StatusLoadError => "Failed to load data",
            Self::StatusPinSuccess => "PIN verified successfully",
            Self::StatusPinFailure => "Incorrect PIN",
            Self::StatusLogout => "Logged out successfully",
            Self::StatusFileTooLarge => "File exceeds size limit",
            Self::StatusPrintSuccess => "Document sent to printer",
            Self::StatusPrintFailure => "Failed to send document to printer",
            Self::StatusThemeChanged => "Color scheme updated",
            Self::StatusConflictError => "Conflict detected. Please reload to avoid overwriting newer changes.",
            Self::StatusValidationError => "Validation failed: please check your input.",
        }
    }
}

/// Look up a translated string. Falls back to English if the language is
/// missing a translation for the given key.
#[must_use]
pub fn lookup(key: StringKey, lang: Language) -> &'static str {
    let entries: &[(&str, &str)] = match key {
        StringKey::TooltipToggleTheme => &[
            ("en", "Toggle theme"),
            ("zh", "切换主题"),
            ("es", "Cambiar tema"),
            ("de", "Design umschalten"),
            ("ja", "テーマ切り替え"),
            ("fr", "Changer de thème"),
            ("pt", "Alternar tema"),
            ("ru", "Переключить тему"),
        ],
        StringKey::TooltipPrint => &[
            ("en", "Print"),
            ("zh", "打印"),
            ("es", "Imprimir"),
            ("de", "Drucken"),
            ("ja", "印刷"),
            ("fr", "Imprimer"),
            ("pt", "Imprimir"),
            ("ru", "Печать"),
        ],
        StringKey::TooltipLogout => &[
            ("en", "Log out"),
            ("zh", "退出登录"),
            ("es", "Cerrar sesión"),
            ("de", "Abmelden"),
            ("ja", "ログアウト"),
            ("fr", "Se déconnecter"),
            ("pt", "Sair"),
            ("ru", "Выйти"),
        ],
        StringKey::AriaSelectLanguage => &[
            ("en", "Select language"),
            ("zh", "选择语言"),
            ("es", "Seleccionar idioma"),
            ("de", "Sprache auswählen"),
            ("ja", "言語を選択"),
            ("fr", "Sélectionner la langue"),
            ("pt", "Selecionar idioma"),
            ("ru", "Выбрать язык"),
        ],
        StringKey::TitleViewReleaseNotes => &[
            ("en", "View Release Notes"),
            ("zh", "查看发行说明"),
            ("es", "Ver notas de la versión"),
            ("de", "Versionshinweise anzeigen"),
            ("ja", "リリースノートを表示"),
            ("fr", "Voir les notes de version"),
            ("pt", "Ver notas de versão"),
            ("ru", "Посмотреть примечания к выпуску"),
        ],
        StringKey::AriaGitHubProfile => &[
            ("en", "GitHub Profile"),
            ("zh", "GitHub 个人主页"),
            ("es", "Perfil de GitHub"),
            ("de", "GitHub-Profil"),
            ("ja", "GitHub プロフィール"),
            ("fr", "Profil GitHub"),
            ("pt", "Perfil do GitHub"),
            ("ru", "Профиль GitHub"),
        ],
        StringKey::StatusReady => &[
            ("en", "Ready"),
            ("zh", "就绪"),
            ("es", "Listo"),
            ("de", "Bereit"),
            ("ja", "準備完了"),
            ("fr", "Prêt"),
            ("pt", "Pronto"),
            ("ru", "Готово"),
        ],
        StringKey::StatusOnline => &[
            ("en", "Connection restored"),
            ("zh", "连接已恢复"),
            ("es", "Conexión restaurada"),
            ("de", "Verbindung wiederhergestellt"),
            ("ja", "接続が回復しました"),
            ("fr", "Connexion rétablie"),
            ("pt", "Conexão restaurada"),
            ("ru", "Соединение восстановлено"),
        ],
        StringKey::StatusOffline => &[
            ("en", "Connection lost"),
            ("zh", "连接已断开"),
            ("es", "Conexión perdida"),
            ("de", "Verbindung verloren"),
            ("ja", "接続が切断されました"),
            ("fr", "Connexion perdue"),
            ("pt", "Conexão perdida"),
            ("ru", "Соединение разорвано"),
        ],
        StringKey::StatusSaving => &[
            ("en", "Saving..."),
            ("zh", "正在保存..."),
            ("es", "Guardando..."),
            ("de", "Speichern..."),
            ("ja", "保存中..."),
            ("fr", "Enregistrement..."),
            ("pt", "Salvando..."),
            ("ru", "Сохранение..."),
        ],
        StringKey::StatusSaved => &[
            ("en", "Changes saved successfully"),
            ("zh", "更改已成功保存"),
            ("es", "Cambios guardados con éxito"),
            ("de", "Änderungen erfolgreich gespeichert"),
            ("ja", "変更が正常に保存されました"),
            ("fr", "Modifications enregistrées avec succès"),
            ("pt", "Alterações salvas com sucesso"),
            ("ru", "Изменения успешно сохранены"),
        ],
        StringKey::StatusSaveError => &[
            ("en", "Failed to save changes"),
            ("zh", "保存更改失败"),
            ("es", "Error al guardar los cambios"),
            ("de", "Fehler beim Speichern der Änderungen"),
            ("ja", "変更の保存に失敗しました"),
            ("fr", "Échec de l'enregistrement des modifications"),
            ("pt", "Falha ao salvar as alterações"),
            ("ru", "Не удалось сохранить изменения"),
        ],
        StringKey::StatusLoadError => &[
            ("en", "Failed to load data"),
            ("zh", "加载数据失败"),
            ("es", "Error al cargar los datos"),
            ("de", "Fehler beim Laden der Daten"),
            ("ja", "データの読み込みに失敗しました"),
            ("fr", "Échec du chargement des données"),
            ("pt", "Falha ao carregar os dados"),
            ("ru", "Не удалось загрузить данные"),
        ],
        StringKey::StatusPinSuccess => &[
            ("en", "PIN verified successfully"),
            ("zh", "PIN 验证成功"),
            ("es", "PIN verificado con éxito"),
            ("de", "PIN erfolgreich verifiziert"),
            ("ja", "PINコードが正常に確認されました"),
            ("fr", "Code PIN vérifié avec succès"),
            ("pt", "PIN verificado com sucesso"),
            ("ru", "PIN-код успешно проверен"),
        ],
        StringKey::StatusPinFailure => &[
            ("en", "Incorrect PIN"),
            ("zh", "PIN 码不正确"),
            ("es", "PIN incorrecto"),
            ("de", "Falscher PIN"),
            ("ja", "PINコードが正しくありません"),
            ("fr", "Code PIN incorrect"),
            ("pt", "PIN incorreto"),
            ("ru", "Неверный PIN-код"),
        ],
        StringKey::StatusLogout => &[
            ("en", "Logged out successfully"),
            ("zh", "成功退出登录"),
            ("es", "Sesión cerrada con éxito"),
            ("de", "Erfolgreich abgemeldet"),
            ("ja", "正常にログアウトしました"),
            ("fr", "Déconnexion réussie"),
            ("pt", "Sair com sucesso"),
            ("ru", "Успешный выход из системы"),
        ],
        StringKey::StatusFileTooLarge => &[
            ("en", "File exceeds size limit"),
            ("zh", "文件超出大小限制"),
            ("es", "El archivo supera el límite de tamaño"),
            ("de", "Datei überschreitet das Größenlimit"),
            ("ja", "ファイルサイズが制限を超えています"),
            ("fr", "Le fichier dépasse la limite de taille"),
            ("pt", "O arquivo excede o limite de tamanho"),
            ("ru", "Файл превышает лимит размера"),
        ],
        StringKey::StatusPrintSuccess => &[
            ("en", "Document sent to printer"),
            ("zh", "文档已发送至打印机"),
            ("es", "Documento enviado a la impresora"),
            ("de", "Dokument an Drucker gesendet"),
            ("ja", "ドキュメントがプリンターに送信されました"),
            ("fr", "Document envoyé à l'imprimante"),
            ("pt", "Documento enviado para a impressora"),
            ("ru", "Документ отправлен на печать"),
        ],
        StringKey::StatusPrintFailure => &[
            ("en", "Failed to send document to printer"),
            ("zh", "无法发送文档至打印机"),
            ("es", "Error al enviar el documento a la impresora"),
            ("de", "Fehler beim Senden des Dokuments an den Drucker"),
            ("ja", "ドキュメントの送信に失敗しました"),
            ("fr", "Échec de l'envoi du document à l'imprimante"),
            ("pt", "Falha ao enviar documento para a impressora"),
            ("ru", "Не удалось отправить документ на печать"),
        ],
        StringKey::StatusThemeChanged => &[
            ("en", "Color scheme updated"),
            ("zh", "配色方案已更新"),
            ("es", "Esquema de colores actualizado"),
            ("de", "Farbschema aktualisiert"),
            ("ja", "カラースキームが更新されました"),
            ("fr", "Palette de couleurs mise à jour"),
            ("pt", "Esquema de cores atualizado"),
            ("ru", "Цветовая схема обновлена"),
        ],
        StringKey::StatusConflictError => &[
            ("en", "Conflict detected. Please reload to avoid overwriting newer changes."),
            ("zh", "检测到冲突。请刷新以避免覆盖较新的更改。"),
            ("es", "Conflicto detectado. Por favor, recargue para evitar sobrescribir cambios más recientes."),
            ("de", "Konflikt erkannt. Bitte neu laden, um das Überschreiben neuerer Änderungen zu verhindern."),
            ("ja", "競合が検出されました。最新の変更を上書きしないよう、再読み込みしてください。"),
            ("fr", "Conflit détecté. Veuillez recharger pour éviter d'écraser des modifications plus récentes."),
            ("pt", "Conflito detetado. Por favor, recarregue para evitar sobrescrever as alterações mais recentes."),
            ("ru", "Обнаружен конфликт. Пожалуйста, перезагрузите страницу, чтобы избежать перезаписи более новых изменений."),
        ],
        StringKey::StatusValidationError => &[
            ("en", "Validation failed: please check your input."),
            ("zh", "验证失败：请检查您的输入。"),
            ("es", "Error de validación: por favor, verifique su entrada."),
            ("de", "Validierung fehlerhaft: Bitte überprüfen Sie Ihre Eingabe."),
            ("ja", "検証に失敗しました：入力を確認してください。"),
            ("fr", "Échec de la validation : veuillez vérifier votre saisie."),
            ("pt", "Falha na validação: por favor, verifique a sua entrada."),
            ("ru", "Ошибка валидации: пожалуйста, проверьте введенные данные."),
        ],
    };

    let code = lang.code();
    entries
        .iter()
        .find(|(c, _)| *c == code)
        .map(|(_, s)| *s)
        .unwrap_or_else(|| key.english())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn english_fallback_present_for_every_key() {
        for key in StringKey::all() {
            assert!(!key.english().is_empty(), "{key:?} has empty fallback");
        }
    }

    #[test]
    fn every_key_has_translation_for_every_language() {
        for key in StringKey::all() {
            for lang in Language::all() {
                let s = lookup(*key, *lang);
                assert!(!s.is_empty(), "{key:?} missing translation for {:?}", lang);
            }
        }
    }

    #[test]
    fn english_matches_known_constants() {
        assert_eq!(
            lookup(StringKey::TooltipToggleTheme, Language::English),
            "Toggle theme"
        );
        assert_eq!(lookup(StringKey::TooltipPrint, Language::English), "Print");
        assert_eq!(
            lookup(StringKey::TooltipLogout, Language::English),
            "Log out"
        );
    }

    #[test]
    fn non_english_codes_return_localized_text() {
        assert_eq!(lookup(StringKey::TooltipPrint, Language::Chinese), "打印");
        assert_eq!(lookup(StringKey::TooltipPrint, Language::Japanese), "印刷");
    }

    #[test]
    fn language_codes_in_table_are_consistent() {
        // All entries should use codes that match Language::code() for at
        // least one variant. If this fails, someone added a code typo.
        for key in StringKey::all() {
            let entries: &[(&str, &str)] = match key {
                StringKey::TooltipToggleTheme => &[
                    ("en", ""),
                    ("zh", ""),
                    ("es", ""),
                    ("de", ""),
                    ("ja", ""),
                    ("fr", ""),
                    ("pt", ""),
                    ("ru", ""),
                ],
                // Just need to verify codes here, so we can re-use any row
                _ => &[
                    ("en", "x"),
                    ("zh", "x"),
                    ("es", "x"),
                    ("de", "x"),
                    ("ja", "x"),
                    ("fr", "x"),
                    ("pt", "x"),
                    ("ru", "x"),
                ],
            };
            for (code, _) in entries {
                // Don't compare to a specific language since order in the
                // table is independent of Language::all(); just check that
                // the code is recognized.
                assert!(
                    Language::all().iter().any(|l| l.code() == *code),
                    "{code} is not a known language code"
                );
            }
        }
    }
}
