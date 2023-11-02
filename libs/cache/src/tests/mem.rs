use std::sync::{Arc, Mutex};

use crossbeam_channel::unbounded;

use crate::error::Error;

use crate::mem::TypeCache;
use crate::tests::Key;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
struct Params1 {
    value: usize,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct Params2 {
    variety: String,
    arc: Arc<Params1>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct Value<T> {
    inner: Arc<T>,
    extra: usize,
}

#[test]
fn generates_in_background_and_caches_values() {
    let mut cache = TypeCache::new();
    let num_gen = Arc::new(Mutex::new(0));
    let (s, r) = unbounded();

    let num_gen_clone = num_gen.clone();

    let params1_func = move |params: &Params1| {
        *num_gen_clone.lock().unwrap() += 1;
        r.recv().unwrap();
        Value {
            inner: Arc::new("substrate".to_string()),
            extra: params.value,
        }
    };

    let p1 = Params1 { value: 5 };

    let expected1 = Value {
        inner: Arc::new("substrate".to_string()),
        extra: 5,
    };

    let handle1 = cache.generate(p1, params1_func.clone());

    // Should not use call the generator as the corresponding block is already being generated.
    let handle2 = cache.generate(p1, params1_func.clone());

    assert!(handle1.poll().is_none());
    assert!(handle2.poll().is_none());

    s.send(()).unwrap();

    assert_eq!(handle1.get(), &expected1);

    // Should reference the same cell as `handle1`.
    assert_eq!(handle2.get(), &expected1);

    // Should immediately return a filled cell as this has already been generated.
    let num_gen_clone = num_gen.clone();
    let handle3 = cache.generate(p1, move |_| {
        *num_gen_clone.lock().unwrap() += 1;
        Value {
            inner: Arc::new("circuit".to_string()),
            extra: 50,
        }
    });

    assert_eq!(handle3.get(), &expected1);

    // Should generate a new block as it has not been generated with the provided parameters
    // yet.
    let handle4 = cache.generate(Params1 { value: 10 }, params1_func);

    s.send(()).unwrap();

    assert_eq!(
        handle4.get(),
        &Value {
            inner: Arc::new("substrate".to_string()),
            extra: 10,
        }
    );

    assert_eq!(*num_gen.lock().unwrap(), 2);
}

#[test]
fn cacheable_api_works() {
    let mut cache = TypeCache::new();
    let handle1 = cache.get(Key(0));
    let handle2 = cache.get(Key(5));
    let handle3 = cache.get(Key(8));

    assert_eq!(*handle1.unwrap_inner(), 0);
    assert_eq!(
        format!("{}", handle2.unwrap_err_inner().root_cause()),
        "invalid key"
    );
    assert!(matches!(handle3.get_err().as_ref(), Error::Panic));

    let state = Arc::new(Mutex::new(Vec::new()));
    let handle1 = cache.get_with_state(Key(0), state.clone());
    let handle2 = cache.get_with_state(Key(5), state.clone());
    let handle3 = cache.get_with_state(Key(8), state.clone());

    assert_eq!(*handle1.unwrap_inner(), 0);
    assert_eq!(
        format!("{}", handle2.unwrap_err_inner().root_cause()),
        "invalid key"
    );
    assert!(matches!(handle3.get_err().as_ref(), Error::Panic));

    assert_eq!(state.lock().unwrap().clone(), vec![0]);
}

#[test]
fn can_cache_multiple_types() {
    let mut cache = TypeCache::new();
    let num_gen = Arc::new(Mutex::new(0));

    let num_gen_clone = num_gen.clone();
    let handle1 = cache.generate(Params1 { value: 5 }, move |_| {
        *num_gen_clone.lock().unwrap() += 1;
        Value {
            inner: Arc::new("substrate".to_string()),
            extra: 20,
        }
    });

    let handle2 = cache.generate(
        Params2 {
            variety: "block".to_string(),
            arc: Arc::new(Params1 { value: 20 }),
        },
        move |_| {
            *num_gen.lock().unwrap() += 1;
            Value {
                inner: Arc::new(5),
                extra: 50,
            }
        },
    );

    assert_eq!(
        handle1.get(),
        &Value {
            inner: Arc::new("substrate".to_string()),
            extra: 20
        }
    );

    assert_eq!(
        handle2.get(),
        &Value {
            inner: Arc::new(5),
            extra: 50
        }
    );
}

#[test]
#[should_panic]
fn panics_on_mismatched_types() {
    let mut cache = TypeCache::new();

    let _ = cache.generate(Params1 { value: 5 }, |_| "cell".to_string());
    let _ = cache.generate(Params1 { value: 10 }, |_| 5);
}

#[test]
fn cache_should_not_hang_on_panic() {
    let mut cache = TypeCache::new();

    let handle = cache.generate::<_, usize>(Params1 { value: 5 }, |_| panic!("generator panicked"));

    assert!(matches!(handle.get_err().as_ref(), Error::Panic));
}
