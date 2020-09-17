// Copyright 2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::error::SummaryError;
use crate::*;
use core::marker::PhantomData;

use rustcommon_atomics::Atomic;
use rustcommon_heatmap::AtomicHeatmap;
use rustcommon_streamstats::AtomicStreamstats;

use core::time::Duration;
use std::time::Instant;

pub(crate) enum SummaryStruct<Value, Count>
where
    Value: crate::Value,
    Count: crate::Count,
    <Value as Atomic>::Primitive: Primitive,
    <Count as Atomic>::Primitive: Primitive,
    u64: From<<Value as Atomic>::Primitive> + From<<Count as Atomic>::Primitive>,
{
    Heatmap(AtomicHeatmap<<Value as Atomic>::Primitive, Count>),
    Stream(AtomicStreamstats<Value>),
}

impl<Value, Count> SummaryStruct<Value, Count>
where
    Value: crate::Value,
    Count: crate::Count,
    <Value as Atomic>::Primitive: Primitive,
    <Count as Atomic>::Primitive: Primitive,
    u64: From<<Value as Atomic>::Primitive> + From<<Count as Atomic>::Primitive>,
{
    pub fn increment(
        &self,
        time: Instant,
        value: <Value as Atomic>::Primitive,
        count: <Count as Atomic>::Primitive,
    ) {
        match self {
            Self::Heatmap(heatmap) => heatmap.increment(time, value, count),
            Self::Stream(stream) => stream.insert(value),
        }
    }

    pub fn percentile(
        &self,
        percentile: f64,
    ) -> Result<<Value as Atomic>::Primitive, SummaryError> {
        match self {
            Self::Heatmap(heatmap) => heatmap
                .percentile(percentile)
                .map_err(|e| SummaryError::from(e)),
            Self::Stream(stream) => stream
                .percentile(percentile)
                .map_err(|e| SummaryError::from(e)),
        }
    }

    pub fn heatmap(
        max: <Value as Atomic>::Primitive,
        precision: u8,
        windows: usize,
        resolution: Duration,
    ) -> Self {
        Self::Heatmap(AtomicHeatmap::new(max, precision, windows, resolution))
    }

    pub fn stream(samples: usize) -> Self {
        Self::Stream(AtomicStreamstats::new(samples))
    }
}

enum SummaryType<Value>
where
    Value: crate::Value,
    <Value as Atomic>::Primitive: Primitive,
    u64: From<<Value as Atomic>::Primitive>,
{
    Heatmap(<Value as Atomic>::Primitive, u8, usize, Duration),
    Stream(usize),
}

pub struct Summary<Value, Count>
where
    Value: crate::Value,
    Count: crate::Count,
    <Value as Atomic>::Primitive: Primitive,
    <Count as Atomic>::Primitive: Primitive,
    u64: From<<Value as Atomic>::Primitive> + From<<Count as Atomic>::Primitive>,
{
    inner: SummaryType<Value>,
    _count: PhantomData<Count>,
}

impl<Value, Count> Summary<Value, Count>
where
    Value: crate::Value,
    Count: crate::Count,
    <Value as Atomic>::Primitive: Primitive,
    <Count as Atomic>::Primitive: Primitive,
    u64: From<<Value as Atomic>::Primitive> + From<<Count as Atomic>::Primitive>,
{
    pub fn heatmap(
        max: <Value as Atomic>::Primitive,
        precision: u8,
        windows: usize,
        resolution: Duration,
    ) -> Summary<Value, Count> {
        Self {
            inner: SummaryType::Heatmap(max, precision, windows, resolution),
            _count: PhantomData,
        }
    }

    pub fn stream(samples: usize) -> Summary<Value, Count> {
        Self {
            inner: SummaryType::Stream(samples),
            _count: PhantomData,
        }
    }

    pub(crate) fn build(&self) -> SummaryStruct<Value, Count> {
        match self.inner {
            SummaryType::Heatmap(max, precision, windows, resolution) => {
                SummaryStruct::heatmap(max, precision, windows, resolution)
            }
            SummaryType::Stream(samples) => SummaryStruct::stream(samples),
        }
    }
}
