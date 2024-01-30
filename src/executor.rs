//! Defines Executor struct.

use crate::{config::Config, Func, FuncRef, Statistics, WasmEdgeResult, WasmValue};
use bit_sys as sys;

/// Defines an execution environment for both pure WASM and compiled WASM.
#[derive(Debug, Clone)]
pub struct Executor {
    pub(crate) inner: sys::Executor,
}
impl Executor {
    /// Creates a new [executor](crate::Executor) to be associated with the given [config](crate::config::Config) and [statistics](crate::Statistics).
    ///
    /// # Arguments
    ///
    /// - `config` specifies the configuration of the new [executor](crate::Executor).
    ///
    /// - `stat` specifies the [statistics](crate::Statistics) needed by the new [executor](crate::Executor).
    ///
    /// # Error
    ///
    /// If fail to create a [executor](crate::Executor), then an error is returned.
    pub fn new(config: Option<&Config>, stat: Option<&mut Statistics>) -> WasmEdgeResult<Self> {
        let inner_executor = match config {
            Some(config) => match stat {
                Some(stat) => sys::Executor::create(Some(&config.inner), Some(&mut stat.inner))?,
                None => sys::Executor::create(Some(&config.inner), None)?,
            },
            None => match stat {
                Some(stat) => sys::Executor::create(None, Some(&mut stat.inner))?,
                None => sys::Executor::create(None, None)?,
            },
        };

        Ok(Self {
            inner: inner_executor,
        })
    }

    /// Runs a host function instance and returns the results.
    ///
    /// # Arguments
    ///
    /// * `func` - The function instance to run.
    ///
    /// * `params` - The arguments to pass to the function.
    ///
    /// # Errors
    ///
    /// If fail to run the host function, then an error is returned.
    pub fn run_func(
        &self,
        func: &Func,
        params: impl IntoIterator<Item = WasmValue>,
    ) -> WasmEdgeResult<Vec<WasmValue>> {
        self.inner.call_func(&func.inner, params)
    }

    /// Runs a host function reference instance and returns the results.
    ///
    /// # Arguments
    ///
    /// * `func_ref` - The function reference instance to run.
    ///
    /// * `params` - The arguments to pass to the function.
    ///
    /// # Errors
    ///
    /// If fail to run the host function reference instance, then an error is returned.
    pub fn run_func_ref(
        &self,
        func_ref: &FuncRef,
        params: impl IntoIterator<Item = WasmValue>,
    ) -> WasmEdgeResult<Vec<WasmValue>> {
        self.inner.call_func_ref(&func_ref.inner, params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::{CommonConfigOptions, ConfigBuilder},
        params, wat2wasm, Module, Statistics, Store, WasmVal,
    };
    #[cfg(all(feature = "async", target_os = "linux"))]
    use crate::{error::HostFuncError, CallingFrame};

    #[test]
    #[allow(clippy::assertions_on_result_states)]
    fn test_executor_create() {
        {
            let result = Executor::new(None, None);
            assert!(result.is_ok());
        }

        {
            let result = ConfigBuilder::new(CommonConfigOptions::default()).build();
            assert!(result.is_ok());
            let config = result.unwrap();

            let result = Executor::new(Some(&config), None);
            assert!(result.is_ok());

            assert!(config.bulk_memory_operations_enabled());
        }

        {
            let result = Statistics::new();
            assert!(result.is_ok());
            let mut stat = result.unwrap();

            let result = Executor::new(None, Some(&mut stat));
            assert!(result.is_ok());

            assert_eq!(stat.cost(), 0);
        }

        {
            let result = ConfigBuilder::new(CommonConfigOptions::default()).build();
            assert!(result.is_ok());
            let config = result.unwrap();

            let result = Statistics::new();
            assert!(result.is_ok());
            let mut stat = result.unwrap();

            let result = Executor::new(Some(&config), Some(&mut stat));
            assert!(result.is_ok());

            assert!(config.bulk_memory_operations_enabled());
            assert_eq!(stat.cost(), 0);
        }
    }

    #[test]
    fn test_executor_run_func() {
        // create an executor
        let result = ConfigBuilder::new(CommonConfigOptions::default()).build();
        assert!(result.is_ok());
        let config = result.unwrap();

        let result = Statistics::new();
        assert!(result.is_ok());
        let mut stat = result.unwrap();

        let result = Executor::new(Some(&config), Some(&mut stat));
        assert!(result.is_ok());
        let mut executor = result.unwrap();

        // create a store
        let result = Store::new();
        assert!(result.is_ok());
        let mut store = result.unwrap();

        // read the wasm bytes of fibonacci.wasm
        let result = wat2wasm(
            br#"
            (module
                (type (;0;) (func (param i32) (result i32)))
                (type (;1;) (func))
                (func (;0;) (type 0) (param i32) (result i32)
                  (local i32 i32 i32)
                  i32.const 1
                  local.set 1
                  block  ;; label = @1
                    local.get 0
                    i32.const 2
                    i32.lt_s
                    br_if 0 (;@1;)
                    local.get 0
                    i32.const -1
                    i32.add
                    local.tee 1
                    i32.const 7
                    i32.and
                    local.set 2
                    block  ;; label = @2
                      block  ;; label = @3
                        local.get 0
                        i32.const -2
                        i32.add
                        i32.const 7
                        i32.ge_u
                        br_if 0 (;@3;)
                        i32.const 1
                        local.set 0
                        i32.const 1
                        local.set 1
                        br 1 (;@2;)
                      end
                      local.get 1
                      i32.const -8
                      i32.and
                      local.set 3
                      i32.const 1
                      local.set 0
                      i32.const 1
                      local.set 1
                      loop  ;; label = @3
                        local.get 1
                        local.get 0
                        i32.add
                        local.tee 0
                        local.get 1
                        i32.add
                        local.tee 1
                        local.get 0
                        i32.add
                        local.tee 0
                        local.get 1
                        i32.add
                        local.tee 1
                        local.get 0
                        i32.add
                        local.tee 0
                        local.get 1
                        i32.add
                        local.tee 1
                        local.get 0
                        i32.add
                        local.tee 0
                        local.get 1
                        i32.add
                        local.set 1
                        local.get 3
                        i32.const -8
                        i32.add
                        local.tee 3
                        br_if 0 (;@3;)
                      end
                    end
                    local.get 2
                    i32.eqz
                    br_if 0 (;@1;)
                    local.get 1
                    local.set 3
                    loop  ;; label = @2
                      local.get 3
                      local.get 0
                      i32.add
                      local.set 1
                      local.get 3
                      local.set 0
                      local.get 1
                      local.set 3
                      local.get 2
                      i32.const -1
                      i32.add
                      local.tee 2
                      br_if 0 (;@2;)
                    end
                  end
                  local.get 1)
                (func (;1;) (type 1))
                (func (;2;) (type 1)
                  call 1
                  call 1)
                (func (;3;) (type 0) (param i32) (result i32)
                  local.get 0
                  call 0
                  call 2)
                (table (;0;) 1 1 funcref)
                (memory (;0;) 16)
                (global (;0;) (mut i32) (i32.const 1048576))
                (export "memory" (memory 0))
                (export "fib" (func 3)))
"#,
        );
        assert!(result.is_ok());
        let wasm_bytes = result.unwrap();
        let result = Module::from_bytes(Some(&config), wasm_bytes);
        assert!(result.is_ok());
        let module = result.unwrap();

        // register a module into store as active module
        let result = store.register_named_module(&mut executor, "extern", &module);
        assert!(result.is_ok());
        let extern_instance = result.unwrap();

        // get the exported function "fib"
        let result = extern_instance.func("fib");
        assert!(result.is_ok());
        let fib = result.unwrap();

        // run the exported host function
        let result = executor.run_func(&fib, params!(5));
        assert!(result.is_ok());
        let returns = result.unwrap();
        assert_eq!(returns.len(), 1);
        assert_eq!(returns[0].to_i32(), 8);
    }

    #[cfg(all(feature = "async", target_os = "linux"))]
    #[tokio::test]
    async fn test_executor_run_async_func() -> Result<(), Box<dyn std::error::Error>> {
        fn async_hello(
            _frame: CallingFrame,
            _inputs: Vec<WasmValue>,
            _data: *mut std::os::raw::c_void,
        ) -> Box<(dyn std::future::Future<Output = Result<Vec<WasmValue>, HostFuncError>> + Send)>
        {
            Box::new(async move {
                for _ in 0..10 {
                    println!("[async hello] say hello");
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }

                println!("[async hello] Done!");

                Ok(vec![])
            })
        }

        #[derive(Debug)]
        struct Data<T, S> {
            _x: i32,
            _y: String,
            _v: Vec<T>,
            _s: Vec<S>,
        }
        let data: Data<i32, &str> = Data {
            _x: 12,
            _y: "hello".to_string(),
            _v: vec![1, 2, 3],
            _s: vec!["macos", "linux", "windows"],
        };

        // create an async host function
        let result =
            Func::wrap_async_func::<(), (), Data<i32, &str>>(async_hello, Some(Box::new(data)));
        assert!(result.is_ok());
        let func = result.unwrap();

        // create an executor
        let executor = Executor::new(None, None).unwrap();

        // create an async state
        let async_state = AsyncState::new();

        async fn tick() {
            let mut i = 0;
            loop {
                println!("[tick] i={i}");
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                i += 1;
            }
        }
        tokio::spawn(tick());

        // call the async host function
        let _ = executor.run_func_async(&async_state, &func, []).await?;

        Ok(())
    }
}
