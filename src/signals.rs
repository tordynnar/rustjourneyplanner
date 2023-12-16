use leptos::*;
use std::future::Future;

pub fn create_memo_local_resource_timed<T, Fu>(
    millis : u64,
    fetcher: impl Fn(Option<T>) -> Fu + Copy + 'static
) -> (Resource<u64, T>, Memo<Option<T>>)
where
    T: Clone + PartialEq + 'static,
    Fu: Future<Output = T> + 'static,
{
    let (refresh, set_refresh) = create_signal(0u64);
    let previous_result = store_value::<Option<T>>(None);

    let resource = create_local_resource(move || { refresh.get() }, async move |_| {
        // This closure is not reentrant. store_value::get_value/set_value
        // call borrow/borrow_mut instead of try_borrow/try_borrow_mut on
        // a RefCell, which means it will panic when called in parallel.

        let result = fetcher(previous_result.get_value()).await;
        previous_result.set_value(Some(result.clone()));
        set_timeout(move || { set_refresh.update(|v| *v += 1); }, std::time::Duration::from_millis(millis));
        result
    });

    let memo = create_memo(move |_|  {
        resource.get()
    });

    (resource, memo)
}