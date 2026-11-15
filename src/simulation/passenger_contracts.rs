//! Fictional passenger charter previews (CON-02). No acceptance, reservation,
//! reward or persistence until a versioned charter transaction exists.
use super::{ServiceState, ServiceWagon, catalog::ContentKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PassengerCharter {
    SchoolVisit,
    ScenicParty,
    DiningTrip,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CharterPreview {
    pub passengers: u32,
    pub free_seats: u32,
    pub missing_seats: u32,
    pub missing_wagon: Option<ServiceWagon>,
}

impl CharterPreview {
    /// Capacity only: this does not imply that a contract was accepted.
    pub fn fits(self) -> bool {
        self.missing_seats == 0 && self.missing_wagon.is_none()
    }
}

impl PassengerCharter {
    pub const ALL: [Self; 3] = [Self::SchoolVisit, Self::ScenicParty, Self::DiningTrip];
    pub const KIND: ContentKind = ContentKind::Fictional;

    pub const fn id(self) -> &'static str {
        match self {
            Self::SchoolVisit => "school-visit",
            Self::ScenicParty => "scenic-party",
            Self::DiningTrip => "dining-trip",
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::SchoolVisit => "Visita de estudantes",
            Self::ScenicParty => "Passeio de observação",
            Self::DiningTrip => "Mesa entre estações",
        }
    }

    pub const fn passengers(self) -> u32 {
        match self {
            Self::SchoolVisit => 12,
            Self::ScenicParty => 8,
            Self::DiningTrip => 6,
        }
    }

    pub const fn required_wagon(self) -> Option<ServiceWagon> {
        match self {
            Self::SchoolVisit => None,
            Self::ScenicParty => Some(ServiceWagon::Observation),
            Self::DiningTrip => Some(ServiceWagon::Dining),
        }
    }

    pub fn preview(self, service: &ServiceState) -> CharterPreview {
        let free_seats = service
            .capacity()
            .0
            .saturating_sub(service.passenger_count());
        CharterPreview {
            passengers: self.passengers(),
            free_seats,
            missing_seats: self.passengers().saturating_sub(free_seats),
            missing_wagon: self
                .required_wagon()
                .filter(|wagon| !service.composition.contains(wagon)),
        }
    }
}
