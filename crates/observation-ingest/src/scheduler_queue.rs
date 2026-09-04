use std::num::NonZeroUsize;

#[must_use]
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CollectionIntentId(String);

impl CollectionIntentId {
    pub fn new(value: impl Into<String>) -> Option<Self> {
        let value = value.into();
        (!value.is_empty()).then_some(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CollectionClass {
    Core,
    Detail,
}

#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkKind {
    Light,
    Heavy,
}

#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompletionDisposition {
    Completed,
    Unknown,
    CollectorRejected,
}

#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SchedulerSafetyLimits {
    pub(crate) queue_capacity: NonZeroUsize,
    pub(crate) max_overrun_debt: NonZeroUsize,
}

impl SchedulerSafetyLimits {
    #[must_use]
    pub fn new(queue_capacity: usize, max_overrun_debt: usize) -> Option<Self> {
        Some(Self {
            queue_capacity: NonZeroUsize::new(queue_capacity)?,
            max_overrun_debt: NonZeroUsize::new(max_overrun_debt)?,
        })
    }
}

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CollectionIntent {
    id: CollectionIntentId,
    class: CollectionClass,
    work_kind: WorkKind,
    declared_work: NonZeroUsize,
    game_time_urgency: u64,
}

impl CollectionIntent {
    #[must_use]
    pub fn new(
        id: CollectionIntentId,
        class: CollectionClass,
        work_kind: WorkKind,
        declared_work: usize,
        game_time_urgency: u64,
    ) -> Option<Self> {
        Some(Self {
            id,
            class,
            work_kind,
            declared_work: NonZeroUsize::new(declared_work)?,
            game_time_urgency,
        })
    }

    pub const fn id(&self) -> &CollectionIntentId {
        &self.id
    }

    pub const fn work_kind(&self) -> WorkKind {
        self.work_kind
    }

    #[must_use]
    pub const fn declared_work(&self) -> usize {
        self.declared_work.get()
    }
}

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SchedulerAdmission {
    pub(crate) intent_id: CollectionIntentId,
    pub(crate) work_kind: WorkKind,
    pub(crate) declared_work: usize,
}

impl SchedulerAdmission {
    pub const fn intent_id(&self) -> &CollectionIntentId {
        &self.intent_id
    }

    pub const fn work_kind(&self) -> WorkKind {
        self.work_kind
    }

    #[must_use]
    pub const fn declared_work(&self) -> usize {
        self.declared_work
    }
}

struct QueuedIntent {
    intent: CollectionIntent,
    ordinal: u64,
}

pub struct IntentQueue {
    capacity: NonZeroUsize,
    next_ordinal: u64,
    queued: Vec<QueuedIntent>,
}

impl IntentQueue {
    pub(super) const fn new(capacity: NonZeroUsize) -> Self {
        Self {
            capacity,
            next_ordinal: 0,
            queued: Vec::new(),
        }
    }

    pub(super) fn enqueue(&mut self, intent: CollectionIntent) -> Result<(), CollectionIntent> {
        if self.queued.len() >= self.capacity.get()
            || self
                .queued
                .iter()
                .any(|queued| queued.intent.id == intent.id)
        {
            return Err(intent);
        }
        let ordinal = self.next_ordinal;
        let Some(next_ordinal) = self.next_ordinal.checked_add(1) else {
            return Err(intent);
        };
        self.next_ordinal = next_ordinal;
        self.queued.push(QueuedIntent { intent, ordinal });
        Ok(())
    }

    pub(super) fn take_best_where(
        &mut self,
        eligible: impl Fn(&CollectionIntent) -> bool,
    ) -> Option<CollectionIntent> {
        let selected = self
            .queued
            .iter()
            .enumerate()
            .filter(|(_, queued)| eligible(&queued.intent))
            .min_by(|(_, left), (_, right)| compare(left, right))
            .map(|(index, _)| index)?;
        Some(self.queued.remove(selected).intent)
    }

    pub(super) const fn len(&self) -> usize {
        self.queued.len()
    }
}

fn compare(left: &QueuedIntent, right: &QueuedIntent) -> std::cmp::Ordering {
    class_rank(left.intent.class)
        .cmp(&class_rank(right.intent.class))
        .then_with(|| {
            right
                .intent
                .game_time_urgency
                .cmp(&left.intent.game_time_urgency)
        })
        .then_with(|| left.ordinal.cmp(&right.ordinal))
}

const fn class_rank(class: CollectionClass) -> u8 {
    match class {
        CollectionClass::Core => 0,
        CollectionClass::Detail => 1,
    }
}
