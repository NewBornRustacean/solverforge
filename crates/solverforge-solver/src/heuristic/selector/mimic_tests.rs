use super::*;
use crate::heuristic::selector::entity::FromSolutionEntitySelector;
use crate::test_utils::create_simple_nqueens_director;

#[test]
fn test_mimic_recording_selector() {
    let director = create_simple_nqueens_director(3);

    let solution = director.working_solution();
    for (i, queen) in solution.queens.iter().enumerate() {
        assert_eq!(queen.column, i as i64);
    }

    let recorder = MimicRecorder::new("test");
    let child = FromSolutionEntitySelector::new(0);
    let recording = MimicRecordingEntitySelector::new(child, recorder);

    let entities: Vec<_> = recording.iter(&director).collect();
    assert_eq!(entities.len(), 3);
    assert_eq!(entities[0], EntityReference::new(0, 0));
    assert_eq!(entities[1], EntityReference::new(0, 1));
    assert_eq!(entities[2], EntityReference::new(0, 2));
}

#[test]
fn test_mimic_replaying_selector() {
    let director = create_simple_nqueens_director(3);

    let recorder = MimicRecorder::new("test");
    let child = FromSolutionEntitySelector::new(0);
    let recording = MimicRecordingEntitySelector::new(child, recorder.clone());
    let replaying = MimicReplayingEntitySelector::new(recorder);

    let mut recording_iter = recording.iter(&director);

    let first = recording_iter.next().unwrap();
    assert_eq!(first, EntityReference::new(0, 0));

    let replayed: Vec<_> = replaying.iter(&director).collect();
    assert_eq!(replayed.len(), 1);
    assert_eq!(replayed[0], EntityReference::new(0, 0));

    let second = recording_iter.next().unwrap();
    assert_eq!(second, EntityReference::new(0, 1));

    let replayed: Vec<_> = replaying.iter(&director).collect();
    assert_eq!(replayed.len(), 1);
    assert_eq!(replayed[0], EntityReference::new(0, 1));
}

#[test]
fn test_mimic_synchronized_iteration() {
    let director = create_simple_nqueens_director(3);

    let recorder = MimicRecorder::new("test");
    let child = FromSolutionEntitySelector::new(0);
    let recording = MimicRecordingEntitySelector::new(child, recorder.clone());
    let replaying = MimicReplayingEntitySelector::new(recorder);

    for recorded in recording.iter(&director) {
        let replayed: Vec<_> = replaying.iter(&director).collect();
        assert_eq!(replayed.len(), 1);
        assert_eq!(replayed[0], recorded);
    }
}

#[test]
fn test_mimic_empty_selector() {
    let director = create_simple_nqueens_director(0);

    let recorder = MimicRecorder::new("test");
    let child = FromSolutionEntitySelector::new(0);
    let recording = MimicRecordingEntitySelector::new(child, recorder.clone());
    let replaying = MimicReplayingEntitySelector::new(recorder);

    let entities: Vec<_> = recording.iter(&director).collect();
    assert_eq!(entities.len(), 0);

    let replayed: Vec<_> = replaying.iter(&director).collect();
    assert_eq!(replayed.len(), 0);
}
