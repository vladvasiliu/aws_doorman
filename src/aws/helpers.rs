use rusoto_ec2::InstanceState;

use crate::aws::error::EC2InstanceError;

pub(in crate::aws) fn is_running(state: &Option<InstanceState>) -> Result<bool, EC2InstanceError> {
    match state {
        Some(InstanceState { code: Some(16), .. }) => Ok(true),
        Some(state) => {
            let code = state.code.unwrap_or_default();
            let name = state.name.as_deref().unwrap_or_default();
            Err(EC2InstanceError::InstanceHasIncorrectState(format!(
                "{} - {}",
                code, name
            )))
        }
        None => Err(EC2InstanceError::InstanceHasIncorrectState(
            "unknown".to_string(),
        )),
    }
}
