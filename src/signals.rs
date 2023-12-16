use leptos::*;
use std::future::Future;
use chrono::{Utc, NaiveDateTime};
use leptos_use::{use_interval, UseIntervalReturn};

#[derive(Debug, Clone)]
pub struct Tracker<E> {
    pub update_time : Option<NaiveDateTime>,
    pub update_error : Option<E>
}

pub fn create_tracked_local_resource<T, E, Fu>(
    millis : u64,
    fetcher: impl Fn(Option<T>) -> Fu + Copy + 'static
) -> (Memo<Option<T>>, Signal<Tracker<E>>)
where
    T: Clone + PartialEq + 'static,
    E: Clone,
    Fu: Future<Output = Result<T, E>> + 'static,
{
    let (refresh, set_refresh) = create_signal(0u64);
    let previous_result = store_value::<(Option<T>,Tracker<E>)>((None, Tracker { update_time : None, update_error: None }));

    let resource = create_local_resource(move || { refresh.get() }, async move |_| {
        // This closure is not reentrant. StoredValue::get_value/set_value
        // call borrow/borrow_mut instead of try_borrow/try_borrow_mut on
        // a RefCell, which means it will panic when called in parallel.

        let (previous_value, previous_tracker) = previous_result.get_value();
        let result = match fetcher(previous_value.clone()).await {
            Ok(v) => (Some(v), Tracker { update_time : Some(Utc::now().naive_utc()), update_error : None }),
            Err(e) => (previous_value, Tracker { update_time : previous_tracker.update_time, update_error : Some(e) })
        };
        previous_result.set_value(result.clone());
        set_timeout(move || { set_refresh.update(|v| *v += 1); }, std::time::Duration::from_millis(millis));
        result
    });

    let memo = create_memo(move |_|  {
        resource.get().map(|(v, _)| v).flatten()
    });

    let UseIntervalReturn { counter : tracker_counter, .. } = use_interval(1000);

    let tracker = Signal::derive(move ||  {
        let _ = tracker_counter.get();
        resource.get().map(|(_, v)| v).unwrap_or(Tracker { update_time : None, update_error: None })
    });

    (memo, tracker)
}