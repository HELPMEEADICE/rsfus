//! Platform-independent safety rules for Rufus operations.

#![no_std]

use core::{error::Error, fmt};

#[cfg(test)]
extern crate std;

/// Rufus currently rejects drives smaller than 8 MiB.
pub const MIN_TARGET_SIZE: u64 = 8 * 1024 * 1024;

/// Rufus permits this margin when comparing an image with its target.
pub const IMAGE_FOOTER_MARGIN: u64 = 4 * 1024;

const UI_DRIVE_INDEX_OFFSET: u32 = 0x80;
const MAX_DRIVES: u32 = 0x40;

/// The unmodified disk number returned by Windows for `PhysicalDriveN`.
///
/// This is deliberately distinct from the C UI's `DRIVE_INDEX_MIN`-offset
/// values.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PhysicalDiskNumber(u32);

impl PhysicalDiskNumber {
    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }

    #[must_use]
    pub const fn device_path(self) -> PhysicalDrivePath {
        PhysicalDrivePath(self)
    }
}

/// Display-only form of a Windows `\\.\PhysicalDriveN` path.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PhysicalDrivePath(PhysicalDiskNumber);

impl fmt::Display for PhysicalDrivePath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, r"\\.\PhysicalDrive{}", self.0.get())
    }
}

/// A physical disk number encoded for storage in the existing C UI controls.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiDriveIndex(u32);

impl UiDriveIndex {
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

impl TryFrom<u32> for UiDriveIndex {
    type Error = DriveIndexError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if (UI_DRIVE_INDEX_OFFSET..UI_DRIVE_INDEX_OFFSET + MAX_DRIVES).contains(&value) {
            Ok(Self(value))
        } else {
            Err(DriveIndexError::InvalidUiIndex(value))
        }
    }
}

impl TryFrom<PhysicalDiskNumber> for UiDriveIndex {
    type Error = DriveIndexError;

    fn try_from(value: PhysicalDiskNumber) -> Result<Self, Self::Error> {
        if value.0 < MAX_DRIVES {
            Ok(Self(value.0 + UI_DRIVE_INDEX_OFFSET))
        } else {
            Err(DriveIndexError::UnsupportedPhysicalDisk(value.0))
        }
    }
}

impl From<UiDriveIndex> for PhysicalDiskNumber {
    fn from(value: UiDriveIndex) -> Self {
        Self(value.0 - UI_DRIVE_INDEX_OFFSET)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DriveIndexError {
    InvalidUiIndex(u32),
    UnsupportedPhysicalDisk(u32),
}

impl fmt::Display for DriveIndexError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidUiIndex(value) => {
                write!(formatter, "invalid Rufus UI drive index: {value}")
            }
            Self::UnsupportedPhysicalDisk(value) => {
                write!(
                    formatter,
                    "physical disk number is outside Rufus range: {value}"
                )
            }
        }
    }
}

impl Error for DriveIndexError {}

/// A snapshot of the properties used to identify a physical target disk.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetDisk<'id> {
    device_number: PhysicalDiskNumber,
    device_instance_id: &'id str,
    disk_size: u64,
    sector_size: u32,
    contains_system_volume: bool,
}

impl<'id> TargetDisk<'id> {
    pub fn new(
        device_number: PhysicalDiskNumber,
        device_instance_id: &'id str,
        disk_size: u64,
        sector_size: u32,
        contains_system_volume: bool,
    ) -> Result<Self, TargetDiskError> {
        if device_instance_id.trim().is_empty() {
            return Err(TargetDiskError::MissingInstanceId);
        }
        if sector_size == 0 {
            return Err(TargetDiskError::InvalidSectorSize);
        }

        Ok(Self {
            device_number,
            device_instance_id,
            disk_size,
            sector_size,
            contains_system_volume,
        })
    }

    #[must_use]
    pub const fn device_number(&self) -> PhysicalDiskNumber {
        self.device_number
    }

    #[must_use]
    pub const fn device_instance_id(&self) -> &'id str {
        self.device_instance_id
    }

    #[must_use]
    pub const fn disk_size(&self) -> u64 {
        self.disk_size
    }

    #[must_use]
    pub const fn sector_size(&self) -> u32 {
        self.sector_size
    }

    #[must_use]
    pub const fn contains_system_volume(&self) -> bool {
        self.contains_system_volume
    }

    fn has_same_identity(&self, other: &TargetDisk<'_>) -> bool {
        self.device_number == other.device_number
            && self.device_instance_id == other.device_instance_id
            && self.disk_size == other.disk_size
            && self.sector_size == other.sector_size
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TargetDiskError {
    MissingInstanceId,
    InvalidSectorSize,
}

impl fmt::Display for TargetDiskError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingInstanceId => formatter.write_str("target disk has no stable instance ID"),
            Self::InvalidSectorSize => {
                formatter.write_str("target disk has an invalid sector size")
            }
        }
    }
}

impl Error for TargetDiskError {}

/// Image information needed before a destructive operation can begin.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceImage {
    projected_size: u64,
    physical_disk_number: Option<PhysicalDiskNumber>,
}

impl SourceImage {
    #[must_use]
    pub const fn new(
        projected_size: u64,
        physical_disk_number: Option<PhysicalDiskNumber>,
    ) -> Self {
        Self {
            projected_size,
            physical_disk_number,
        }
    }
}

/// An immutable plan created before Rufus asks the user for final confirmation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WritePlan<'id> {
    target: TargetDisk<'id>,
    source: Option<SourceImage>,
}

impl<'id> WritePlan<'id> {
    #[must_use]
    pub const fn new(target: TargetDisk<'id>, source: Option<SourceImage>) -> Self {
        Self { target, source }
    }
}

/// Proof that the target was revalidated immediately before destructive I/O.
///
/// The private fields prevent callers from constructing this value without
/// passing [`authorize_write`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WriteAuthorization<'id> {
    target: TargetDisk<'id>,
}

impl<'id> WriteAuthorization<'id> {
    #[must_use]
    pub const fn target(&self) -> &TargetDisk<'id> {
        &self.target
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WriteRejection {
    TargetChanged,
    SystemDisk,
    TargetTooSmall,
    SourceOnTarget,
    ImageTooLarge,
    ConfirmationMissing,
}

impl fmt::Display for WriteRejection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TargetChanged => formatter.write_str("the selected target disk changed"),
            Self::SystemDisk => formatter.write_str("the target contains a system volume"),
            Self::TargetTooSmall => formatter.write_str("the target disk is too small"),
            Self::SourceOnTarget => formatter.write_str("the source image is on the target disk"),
            Self::ImageTooLarge => {
                formatter.write_str("the source image does not fit on the target")
            }
            Self::ConfirmationMissing => formatter.write_str("destructive write was not confirmed"),
        }
    }
}

impl Error for WriteRejection {}

/// Revalidate all safety properties immediately before destructive disk I/O.
pub fn authorize_write<'id>(
    plan: &WritePlan<'_>,
    observed_target: &TargetDisk<'id>,
    user_confirmed: bool,
) -> Result<WriteAuthorization<'id>, WriteRejection> {
    if !plan.target.has_same_identity(observed_target) {
        return Err(WriteRejection::TargetChanged);
    }
    if plan.target.contains_system_volume || observed_target.contains_system_volume {
        return Err(WriteRejection::SystemDisk);
    }
    if observed_target.disk_size < MIN_TARGET_SIZE {
        return Err(WriteRejection::TargetTooSmall);
    }
    if plan
        .source
        .is_some_and(|source| source.physical_disk_number == Some(observed_target.device_number))
    {
        return Err(WriteRejection::SourceOnTarget);
    }
    if plan.source.is_some_and(|source| {
        source.projected_size
            > observed_target
                .disk_size
                .saturating_add(IMAGE_FOOTER_MARGIN)
    }) {
        return Err(WriteRejection::ImageTooLarge);
    }
    if !user_confirmed {
        return Err(WriteRejection::ConfirmationMissing);
    }

    Ok(WriteAuthorization {
        target: observed_target.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const DISK_SIZE: u64 = 16 * 1024 * 1024;

    fn disk() -> TargetDisk<'static> {
        TargetDisk::new(
            PhysicalDiskNumber::new(7),
            "USB\\VID_1234&PID_5678\\SERIAL",
            DISK_SIZE,
            512,
            false,
        )
        .expect("fixture must be valid")
    }

    fn plan(source: Option<SourceImage>) -> WritePlan<'static> {
        WritePlan::new(disk(), source)
    }

    #[test]
    fn authorizes_revalidated_confirmed_target() {
        let target = disk();
        let authorization = authorize_write(
            &plan(Some(SourceImage::new(
                DISK_SIZE,
                Some(PhysicalDiskNumber::new(9)),
            ))),
            &target,
            true,
        )
        .expect("safe write should be authorized");

        assert_eq!(authorization.target(), &target);
    }

    #[test]
    fn rejects_changed_target_properties() {
        let selected = disk();
        let changed_targets = [
            TargetDisk::new(
                PhysicalDiskNumber::new(8),
                selected.device_instance_id(),
                DISK_SIZE,
                512,
                false,
            )
            .expect("fixture must be valid"),
            TargetDisk::new(
                PhysicalDiskNumber::new(7),
                "DIFFERENT_INSTANCE",
                DISK_SIZE,
                512,
                false,
            )
            .expect("fixture must be valid"),
            TargetDisk::new(
                PhysicalDiskNumber::new(7),
                selected.device_instance_id(),
                DISK_SIZE * 2,
                512,
                false,
            )
            .expect("fixture must be valid"),
            TargetDisk::new(
                PhysicalDiskNumber::new(7),
                selected.device_instance_id(),
                DISK_SIZE,
                4096,
                false,
            )
            .expect("fixture must be valid"),
        ];
        let write_plan = WritePlan::new(selected, None);

        for changed_target in changed_targets {
            assert_eq!(
                authorize_write(&write_plan, &changed_target, true),
                Err(WriteRejection::TargetChanged)
            );
        }
    }

    #[test]
    fn rejects_system_disk_even_if_status_changed_after_selection() {
        let observed = TargetDisk::new(
            PhysicalDiskNumber::new(7),
            disk().device_instance_id(),
            DISK_SIZE,
            512,
            true,
        )
        .expect("fixture must be valid");

        assert_eq!(
            authorize_write(&plan(None), &observed, true),
            Err(WriteRejection::SystemDisk)
        );
    }

    #[test]
    fn rejects_source_image_on_target() {
        assert_eq!(
            authorize_write(
                &plan(Some(SourceImage::new(
                    DISK_SIZE,
                    Some(PhysicalDiskNumber::new(7)),
                ))),
                &disk(),
                true
            ),
            Err(WriteRejection::SourceOnTarget)
        );
    }

    #[test]
    fn rejects_target_smaller_than_existing_rufus_limit() {
        let target = TargetDisk::new(
            PhysicalDiskNumber::new(7),
            "SMALL_DISK",
            MIN_TARGET_SIZE - 1,
            512,
            false,
        )
        .expect("fixture must be valid");

        assert_eq!(
            authorize_write(&WritePlan::new(target.clone(), None), &target, true),
            Err(WriteRejection::TargetTooSmall)
        );
    }

    #[test]
    fn applies_existing_image_footer_margin() {
        let target = disk();
        let accepted = SourceImage::new(DISK_SIZE + IMAGE_FOOTER_MARGIN, None);
        let rejected = SourceImage::new(DISK_SIZE + IMAGE_FOOTER_MARGIN + 1, None);

        assert!(authorize_write(&plan(Some(accepted)), &target, true).is_ok());
        assert_eq!(
            authorize_write(&plan(Some(rejected)), &target, true),
            Err(WriteRejection::ImageTooLarge)
        );
    }

    #[test]
    fn requires_final_user_confirmation() {
        assert_eq!(
            authorize_write(&plan(None), &disk(), false),
            Err(WriteRejection::ConfirmationMissing)
        );
    }

    #[test]
    fn rejects_invalid_target_snapshots() {
        assert_eq!(
            TargetDisk::new(PhysicalDiskNumber::new(7), " ", DISK_SIZE, 512, false),
            Err(TargetDiskError::MissingInstanceId)
        );
        assert_eq!(
            TargetDisk::new(PhysicalDiskNumber::new(7), "INSTANCE", DISK_SIZE, 0, false),
            Err(TargetDiskError::InvalidSectorSize)
        );
    }

    #[test]
    fn converts_all_supported_physical_disk_numbers_to_ui_indexes() {
        for physical_number in 0..MAX_DRIVES {
            let physical = PhysicalDiskNumber::new(physical_number);
            let ui_index = UiDriveIndex::try_from(physical).expect("supported disk must convert");

            assert_eq!(ui_index.get(), UI_DRIVE_INDEX_OFFSET + physical_number);
            assert_eq!(UiDriveIndex::try_from(ui_index.get()), Ok(ui_index));
            assert_eq!(PhysicalDiskNumber::from(ui_index), physical);
        }
    }

    #[test]
    fn rejects_values_outside_the_existing_drive_table() {
        assert_eq!(
            UiDriveIndex::try_from(UI_DRIVE_INDEX_OFFSET - 1),
            Err(DriveIndexError::InvalidUiIndex(UI_DRIVE_INDEX_OFFSET - 1))
        );
        assert_eq!(
            UiDriveIndex::try_from(UI_DRIVE_INDEX_OFFSET + MAX_DRIVES),
            Err(DriveIndexError::InvalidUiIndex(
                UI_DRIVE_INDEX_OFFSET + MAX_DRIVES
            ))
        );
        assert_eq!(
            UiDriveIndex::try_from(PhysicalDiskNumber::new(MAX_DRIVES)),
            Err(DriveIndexError::UnsupportedPhysicalDisk(MAX_DRIVES))
        );
    }

    #[test]
    fn formats_the_existing_windows_physical_drive_path() {
        assert_eq!(
            std::format!("{}", PhysicalDiskNumber::new(7).device_path()),
            r"\\.\PhysicalDrive7"
        );
    }
}
