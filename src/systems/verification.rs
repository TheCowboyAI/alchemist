//! Verification systems

use crate::components::{IdentityEntity, IdentityVerification};
use crate::value_objects::{VerificationLevel, VerificationMethod};
use bevy::prelude::*;
use std::time::SystemTime;

/// Start verification system
pub fn start_verification_system(
    mut verification_events: EventReader<StartVerificationEvent>,
    mut identities: Query<(&IdentityEntity, &mut IdentityVerification)>,
    mut pending_events: EventWriter<VerificationPendingEvent>,
) {
    for event in verification_events.read() {
        if let Some((_entity, mut verification)) = identities
            .iter_mut()
            .find(|(e, _)| e.id == event.identity_id)
        {
            // Set verification method
            verification.verification_method = Some(event.method.clone());

            // Emit pending event
            pending_events.write(VerificationPendingEvent {
                identity_id: event.identity_id,
                method: event.method.clone(),
                started_at: SystemTime::now(),
            });

            info!(
                "Started verification for identity {} using {:?}",
                event.identity_id, event.method
            );
        }
    }
}

/// Process verification system
pub fn process_verification_system(
    mut process_events: EventReader<ProcessVerificationEvent>,
    mut identities: Query<(&IdentityEntity, &mut IdentityVerification)>,
    mut complete_events: EventWriter<VerificationCompleteEvent>,
) {
    for event in process_events.read() {
        if let Some((_entity, mut verification)) = identities
            .iter_mut()
            .find(|(e, _)| e.id == event.identity_id)
        {
            // Update verification based on result
            if event.success {
                // Upgrade verification level based on method
                let new_level = match &event.method {
                    VerificationMethod::Email => VerificationLevel::Basic,
                    VerificationMethod::Phone => VerificationLevel::Basic,
                    VerificationMethod::Document => VerificationLevel::Advanced,
                    VerificationMethod::Biometric => VerificationLevel::Full,
                };

                // Only upgrade, never downgrade
                if new_level > verification.verification_level {
                    verification.verification_level = new_level;
                }

                verification.last_verified = Some(SystemTime::now());

                complete_events.write(VerificationCompleteEvent {
                    identity_id: event.identity_id,
                    new_level: verification.verification_level.clone(),
                    verified_at: SystemTime::now(),
                });

                info!("Verification successful for identity {}", event.identity_id);
            } else {
                warn!("Verification failed for identity {}", event.identity_id);
            }
        }
    }
}

/// Complete verification system
pub fn complete_verification_system(
    mut complete_events: EventReader<VerificationCompleteEvent>,
    identities: Query<(&IdentityEntity, &IdentityVerification)>,
) {
    for event in complete_events.read() {
        // Find the verified identity to validate the completion
        if let Some((_entity, verification)) =
            identities.iter().find(|(e, _)| e.id == event.identity_id)
        {
            // Verify that the verification level matches what we expect
            if verification.verification_level != event.new_level {
                warn!(
                    "Verification level mismatch for identity {}: expected {:?}, found {:?}",
                    event.identity_id, event.new_level, verification.verification_level
                );
            }

            // Log successful completion with actual data
            info!(
                "Verification completed for identity {} with level {:?}, last verified: {:?}",
                event.identity_id, verification.verification_level, verification.last_verified
            );

            // In production, you might:
            // - Send notifications
            // - Update access permissions based on new verification level
            // - Trigger dependent workflows
            // - Update audit logs
        } else {
            error!(
                "Verification completed for non-existent identity {}",
                event.identity_id
            );
        }
    }
}

/// Expire verifications system
pub fn expire_verifications_system(
    time: Res<Time>,
    mut identities: Query<(&IdentityEntity, &mut IdentityVerification)>,
    mut expired_events: EventWriter<VerificationExpiredEvent>,
) {
    // Check every 60 seconds
    let check_interval = 60.0;
    let elapsed = time.elapsed_secs();

    if elapsed % check_interval > time.delta_secs() {
        return;
    }

    let now = SystemTime::now();
    let expiry_duration = std::time::Duration::from_secs(90 * 24 * 60 * 60); // 90 days

    for (entity, mut verification) in &mut identities {
        if let Some(last_verified) = verification.last_verified {
            if let Ok(duration) = now.duration_since(last_verified) {
                if duration > expiry_duration {
                    // Downgrade verification level
                    verification.verification_level = VerificationLevel::None;
                    verification.last_verified = None;

                    expired_events.write(VerificationExpiredEvent {
                        identity_id: entity.id,
                        expired_at: now,
                    });

                    warn!("Verification expired for identity {}", entity.id);
                }
            }
        }
    }
}

// Event types for verification systems
/// Event to start verification
#[derive(Event)]
pub struct StartVerificationEvent {
    /// Identity to verify
    pub identity_id: uuid::Uuid,
    /// Verification method
    pub method: VerificationMethod,
}

/// Event when verification is pending
#[derive(Event)]
pub struct VerificationPendingEvent {
    /// Identity being verified
    pub identity_id: uuid::Uuid,
    /// Method being used
    pub method: VerificationMethod,
    /// When verification started
    pub started_at: SystemTime,
}

/// Event to process verification result
#[derive(Event)]
pub struct ProcessVerificationEvent {
    /// Identity being verified
    pub identity_id: uuid::Uuid,
    /// Method used
    pub method: VerificationMethod,
    /// Whether verification succeeded
    pub success: bool,
}

/// Event when verification is complete
#[derive(Event)]
pub struct VerificationCompleteEvent {
    /// Identity that was verified
    pub identity_id: uuid::Uuid,
    /// New verification level
    pub new_level: VerificationLevel,
    /// When verification completed
    pub verified_at: SystemTime,
}

/// Event when verification expires
#[derive(Event)]
pub struct VerificationExpiredEvent {
    /// Identity whose verification expired
    pub identity_id: uuid::Uuid,
    /// When it expired
    pub expired_at: SystemTime,
}
