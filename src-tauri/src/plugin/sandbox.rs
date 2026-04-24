use super::{Permission, PluginError};

/// Result of a permission check.
#[derive(Debug, Clone, PartialEq)]
pub enum PermissionCheck {
    Allowed,
    Denied { reason: String },
}

/// Check if a plugin with the given declared permissions is allowed to perform an operation.
///
/// Phase 1: declarative check only — verifies the operation is in the declared set.
/// Phase 2+: will integrate runtime interception.
pub fn check_permission(
    declared_permissions: &[Permission],
    required: &Permission,
) -> PermissionCheck {
    if declared_permissions.contains(required) {
        PermissionCheck::Allowed
    } else {
        PermissionCheck::Denied {
            reason: format!(
                "plugin does not declare required permission: {:?}",
                required
            ),
        }
    }
}

/// Convenience wrapper that returns `Ok(())` or an error.
pub fn require_permission(
    declared_permissions: &[Permission],
    required: &Permission,
) -> Result<(), PluginError> {
    match check_permission(declared_permissions, required) {
        PermissionCheck::Allowed => Ok(()),
        PermissionCheck::Denied { reason } => {
            Err(PluginError::PermissionDenied(reason))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allow_declared_permission() {
        let declared = vec![Permission::FileWrite, Permission::FileRead];
        assert_eq!(
            check_permission(&declared, &Permission::FileWrite),
            PermissionCheck::Allowed
        );
        assert_eq!(
            check_permission(&declared, &Permission::FileRead),
            PermissionCheck::Allowed
        );
    }

    #[test]
    fn test_deny_undeclared_permission() {
        let declared = vec![Permission::FileRead];
        let result = check_permission(&declared, &Permission::Network);
        assert_eq!(result, PermissionCheck::Denied { reason: "plugin does not declare required permission: Network".to_string() });
    }

    #[test]
    fn test_deny_when_no_permissions() {
        let declared: Vec<Permission> = vec![];
        let result = check_permission(&declared, &Permission::FileWrite);
        assert_eq!(result, PermissionCheck::Denied { reason: "plugin does not declare required permission: FileWrite".to_string() });
    }

    #[test]
    fn test_require_permission_returns_ok() {
        let declared = vec![Permission::Network];
        assert!(require_permission(&declared, &Permission::Network).is_ok());
    }

    #[test]
    fn test_require_permission_returns_error() {
        let declared = vec![];
        let result = require_permission(&declared, &Permission::Network);
        assert!(matches!(result, Err(PluginError::PermissionDenied(_))));
    }
}
