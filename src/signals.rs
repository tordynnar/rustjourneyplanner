use leptos::*;
use std::future::Future;

pub fn create_local_resource_timed<T, Fu>(
    millis : u64,
    fetcher: impl Fn() -> Fu + 'static + Copy
) -> Resource<u64, T>
where
    T: 'static,
    Fu: Future<Output = T> + 'static,
{
    let (refresh, set_refresh) = create_signal(0u64);

    create_local_resource(move || { refresh.get() }, move |_| async move {
        let result = fetcher().await;
        set_timeout(move || { set_refresh.update(|v| *v += 1); }, std::time::Duration::from_millis(millis));
        result
    })
}