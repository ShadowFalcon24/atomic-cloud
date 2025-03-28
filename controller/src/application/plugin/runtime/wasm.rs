use std::sync::Arc;

use anyhow::{anyhow, Result};
use common::error::FancyError;
use generated::exports::plugin::system::bridge;
use node::PluginNode;
use tokio::{spawn, sync::Mutex, task::JoinHandle};
use tonic::async_trait;
use url::Url;
use wasmtime::{component::ResourceAny, AsContextMut, Engine, Store};
use wasmtime_wasi::{IoView, ResourceTable, WasiCtx, WasiView};
use wasmtime_wasi_http::{WasiHttpCtx, WasiHttpView};

use crate::application::{
    global::GlobalData,
    node::Capabilities,
    plugin::{BoxedNode, Features, GenericPlugin, Information},
};

pub(crate) mod config;
mod epoch;
pub mod ext;
pub mod init;
mod node;

#[allow(clippy::all)]
pub mod generated {
    use wasmtime::component::bindgen;

    bindgen!({
        world: "plugin",
        path: "../protocol/wit/",
        async: true,
        trappable_imports: true,
        with: {
            "plugin:system/guard/guard": crate::application::server::guard::Guard,
            "plugin:system/process/process-builder": super::ext::process::ProcessBuilder,
            "plugin:system/process/process": super::ext::process::Process,
        }
    });
}

pub(crate) struct PluginState {
    /* Global */
    global: Arc<GlobalData>,

    /* Plugin */
    name: String,

    /* Wasmtime */
    wasi: WasiCtx,
    http: WasiHttpCtx,
    resources: ResourceTable,
}

pub(crate) struct Plugin {
    dropped: bool,

    #[allow(unused)]
    engine: Engine,
    bindings: Arc<generated::Plugin>,
    store: Arc<Mutex<Store<PluginState>>>,
    instance: ResourceAny,
}

#[async_trait]
impl GenericPlugin for Plugin {
    async fn init(&self) -> Result<Information> {
        let (bindings, store, instance) = self.get();
        let mut store = store.lock().await;
        match bindings
            .plugin_system_bridge()
            .plugin()
            .call_init(store.as_context_mut(), instance)
            .await
        {
            Ok(information) => Ok(information.into()),
            Err(error) => Err(error),
        }
    }

    async fn init_node(
        &self,
        name: &str,
        capabilities: &Capabilities,
        controller: &Url,
    ) -> Result<BoxedNode> {
        let (bindings, store, instance) = self.get();
        match bindings
            .plugin_system_bridge()
            .plugin()
            .call_init_node(
                store.clone().lock().await.as_context_mut(),
                instance,
                name,
                &capabilities.into(),
                controller.as_ref(),
            )
            .await?
        {
            Ok(instance) => Ok(Box::new(PluginNode::new(bindings, store, instance)) as BoxedNode),
            Err(error) => Err(anyhow!(error)),
        }
    }

    fn shutdown(&self) -> JoinHandle<Result<()>> {
        let (bindings, store, instance) = self.get();
        spawn(async move {
            match bindings
                .plugin_system_bridge()
                .plugin()
                .call_shutdown(store.lock().await.as_context_mut(), instance)
                .await
            {
                Ok(result) => result.map_err(|errors| {
                    anyhow!(errors
                        .iter()
                        .map(|error| format!("Scope: {}, Message: {}", error.scope, error.message))
                        .collect::<Vec<_>>()
                        .join("\n"))
                }),
                Err(error) => Err(error),
            }
        })
    }

    fn tick(&self) -> JoinHandle<Result<()>> {
        let (bindings, store, instance) = self.get();
        spawn(async move {
            match bindings
                .plugin_system_bridge()
                .plugin()
                .call_tick(store.lock().await.as_context_mut(), instance)
                .await
            {
                Ok(result) => result.map_err(|errors| {
                    anyhow!(errors
                        .iter()
                        .map(|error| format!("Scope: {}, Message: {}", error.scope, error.message))
                        .collect::<Vec<_>>()
                        .join("\n"))
                }),
                Err(error) => Err(error),
            }
        })
    }

    async fn cleanup(&mut self) -> Result<()> {
        self.instance
            .resource_drop_async(self.store.lock().await.as_context_mut())
            .await?;
        self.dropped = true;

        Ok(())
    }
}

impl Drop for Plugin {
    fn drop(&mut self) {
        if !self.dropped {
            FancyError::print_fancy(
                &anyhow!("Resource was not dropped before being deallocated (memory leak)"),
                false,
            );
        }
    }
}

impl Plugin {
    fn get(
        &self,
    ) -> (
        Arc<generated::Plugin>,
        Arc<Mutex<Store<PluginState>>>,
        ResourceAny,
    ) {
        (self.bindings.clone(), self.store.clone(), self.instance)
    }
}

impl IoView for PluginState {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.resources
    }
}

impl WasiView for PluginState {
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi
    }
}

impl WasiHttpView for PluginState {
    fn ctx(&mut self) -> &mut WasiHttpCtx {
        &mut self.http
    }
}

impl From<bridge::Information> for Information {
    fn from(val: bridge::Information) -> Self {
        Information {
            authors: val.authors,
            version: val.version,
            features: val.features.into(),
            ready: val.ready,
        }
    }
}

impl From<bridge::Features> for Features {
    fn from(value: bridge::Features) -> Self {
        let mut features = Features::empty();
        if value.contains(bridge::Features::NODE) {
            features.insert(Features::NODE);
        }
        features
    }
}

impl From<&Capabilities> for bridge::Capabilities {
    fn from(val: &Capabilities) -> Self {
        bridge::Capabilities {
            memory: *val.memory(),
            max_servers: *val.max_servers(),
            child: val.child().as_ref().map(std::string::ToString::to_string),
        }
    }
}
