use wasmtime::component::bindgen;
use wasmtime_wasi::{ResourceTable, WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView};

use crate::api::host::{Color, Host, Pos};

// auto-generated API from WIT
bindgen!("pathfinding" in "wit/world.wit");

#[derive(Debug, Clone, Copy)]
pub enum TimelineAction {
    Tile { pos: Pos, color: Color },
    Line { start: Pos, end: Pos, color: Color },
    Arrow { start: Pos, end: Pos, color: Color },
}

pub struct WasmRunner {
    pub wasi_ctx: WasiCtx,             // For WASI
    pub table: ResourceTable,          // For WASI
    pub timeline: Vec<TimelineAction>, // For pathfinding API
}

impl Default for WasmRunner {
    fn default() -> Self {
        WasmRunner {
            wasi_ctx: WasiCtxBuilder::new()
                .inherit_stdin()
                .inherit_stdout()
                .build(),
            table: ResourceTable::new(),
            timeline: Vec::new(),
        }
    }
}

impl WasiView for WasmRunner {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi_ctx,
            table: &mut self.table,
        }
    }
}

impl Host for WasmRunner {
    fn tile(&mut self, pos: Pos, color: Color) {
        self.timeline.push(TimelineAction::Tile { pos, color });
    }

    fn line(&mut self, start: Pos, end: Pos, color: Color) {
        self.timeline
            .push(TimelineAction::Line { start, end, color });
    }

    fn arrow(&mut self, start: Pos, end: Pos, color: Color) {
        self.timeline
            .push(TimelineAction::Arrow { start, end, color });
    }

    fn output(&mut self, path: Vec<(u32, u32)>) -> bool {
        path.windows(2).for_each(|window| {
            if let [start, end] = window {
                self.line(*start, *end, (0, 200, 0));
            }
        });
        true
    }
}
