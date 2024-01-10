pub mod utils;

pub fn select_from_list<T, F>(list: &[T], disp_f: F) -> usize
where
    F: Fn(&T) -> String,
{
    for el in list.iter() {
        disp_f(el);
    }
    // TODO    TUI interface to select an element from a list
    0
}
