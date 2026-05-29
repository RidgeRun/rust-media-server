/*
 * Copyright 2026 RidgeRun, LLC (http://www.ridgerun.com)
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are
 * met:
 *
 * 1. Redistributions of source code must retain the above copyright
 * notice, this list of conditions and the following disclaimer.
 *
 * 2. Redistributions in binary form must reproduce the above copyright
 * notice, this list of conditions and the following disclaimer in the
 * documentation and/or other materials provided with the distribution.
 *
 * 3. Neither the name of the copyright holder nor the names of its
 * contributors may be used to endorse or promote products derived from
 * this software without specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
 * "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
 * LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
 * A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
 * HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
 * SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
 * LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
 * DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
 * THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */

use std::ptr;
use std::thread::{self, JoinHandle};

use glib::MainLoop;
use gstc::Client;

use crate::common::errors::{AppResult, ErrorCode};
use crate::ports::IPipeline;

mod gstd {
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(non_upper_case_globals)]
    #![allow(dead_code)]
    #![allow(clippy::all)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub struct GstcManager {
    gstd: *mut gstd::GstD,
    gstc: Client,
    main_loop: MainLoop,
    main_loop_thread: Option<JoinHandle<()>>,
}

impl GstcManager {
    pub fn new(address: &str, gstc_port: &i32) -> AppResult<Self> {
        const WAIT_TIME_MS: i32 = 50_000;
        const KEEP_COMMS: bool = true;

        let main_loop = MainLoop::new(None, false);
        let main_loop_runner = main_loop.clone();
        let main_loop_thread = thread::spawn(move || {
            main_loop_runner.run();
        });

        let mut gstd_ptr: *mut gstd::GstD = ptr::null_mut();

        unsafe {
            gstd::gstd_new(&mut gstd_ptr, 0, ptr::null_mut());
        }

        if gstd_ptr.is_null() {
            main_loop.quit();
            let _ = main_loop_thread.join();
            return Err(ErrorCode::KGstCError);
        }

        let gstd_started = unsafe { gstd::gstd_start(gstd_ptr) };
        if gstd_started == 0 {
            unsafe {
                gstd::gstd_free(gstd_ptr);
            }
            main_loop.quit();
            let _ = main_loop_thread.join();
            return Err(ErrorCode::KGstCError);
        }

        let gstc = match Client::new(address, *gstc_port as u16, WAIT_TIME_MS, KEEP_COMMS) {
            Ok(client) => client,
            Err(_) => {
                unsafe {
                    gstd::gstd_stop(gstd_ptr);
                    gstd::gstd_free(gstd_ptr);
                }
                main_loop.quit();
                let _ = main_loop_thread.join();
                return Err(ErrorCode::KGstCError);
            }
        };

        Ok(Self {
            gstd: gstd_ptr,
            gstc,
            main_loop,
            main_loop_thread: Some(main_loop_thread),
        })
    }
}

impl IPipeline for GstcManager {
    fn create_pipeline(
        &mut self,
        pipeline_name: &str,
        pipeline_description: &str,
    ) -> AppResult<()> {
        self.gstc
            .pipeline_create(pipeline_name, pipeline_description)
            .map_err(|_| ErrorCode::KGstCError)
    }

    fn play_pipeline(&mut self, pipeline_name: &str) -> AppResult<()> {
        self.gstc
            .pipeline_play(pipeline_name)
            .map_err(|_| ErrorCode::KGstCError)
    }

    fn stop_pipeline(&mut self, pipeline_name: &str) -> AppResult<()> {
        self.gstc
            .pipeline_stop(pipeline_name)
            .map_err(|_| ErrorCode::KGstCError)
    }

    fn pause_pipeline(&mut self, pipeline_name: &str) -> AppResult<()> {
        self.gstc
            .pipeline_pause(pipeline_name)
            .map_err(|_| ErrorCode::KGstCError)
    }

    fn delete_pipeline(&mut self, pipeline_name: &str) -> AppResult<()> {
        self.gstc
            .pipeline_delete(pipeline_name)
            .map_err(|_| ErrorCode::KGstCError)
    }

    fn set_element_property(
        &mut self,
        pipeline_name: &str,
        element_name: &str,
        property_name: &str,
        value: &str,
    ) -> AppResult<()> {
        self.gstc
            .element_set(pipeline_name, element_name, property_name, value)
            .map_err(|_| ErrorCode::KGstCError)
    }

    fn set_element_property_int(
        &mut self,
        pipeline_name: &str,
        element_name: &str,
        property_name: &str,
        value: i32,
    ) -> AppResult<()> {
        self.gstc
            .element_set(
                pipeline_name,
                element_name,
                property_name,
                &value.to_string(),
            )
            .map_err(|_| ErrorCode::KGstCError)
    }

    fn get_state(&self, pipeline_name: &str) -> AppResult<String> {
        self.gstc
            .pipeline_get_state(pipeline_name)
            .map_err(|_| ErrorCode::KGstCError)
    }
}

impl Drop for GstcManager {
    fn drop(&mut self) {
        self.main_loop.quit();

        if let Some(thread) = self.main_loop_thread.take() {
            let _ = thread.join();
        }

        unsafe {
            gstd::gstd_stop(self.gstd);
            gstd::gstd_free(self.gstd);
        }
    }
}
