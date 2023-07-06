//use crossbeam::atomic::AtomicCell;
use atomic_float::AtomicF32;
use nih_plug::prelude::{util, Editor};
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::widgets::*;
use nih_plug_vizia::{assets, create_vizia_editor, ViziaState, ViziaTheming};
use std::sync::atomic::{Ordering};
use std::sync::Arc;
use std::time::Duration;

use crate::GainParams;

#[derive(Clone, Lens)]
struct Data {
    /// Determines which parts of the GUI are visible, and in turn decides the GUI's size.
    params: Arc<GainParams>,
    in_meter: Arc<AtomicF32>,
    out_meter: Arc<AtomicF32>,
}

impl Model for Data {}

// Makes sense to also define this here, makes it a bit easier to keep track of
pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::new(|| (380, 320))
}

pub(crate) fn create(
    params: Arc<GainParams>,
    in_meter: Arc<AtomicF32>,
    out_meter: Arc<AtomicF32>,
    editor_state: Arc<ViziaState>,
) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, ViziaTheming::Custom, move |cx, _| {
        assets::register_noto_sans_light(cx);
        //assets::register_noto_sans_thin(cx);

        Data {
            params: params.clone(),
            in_meter: in_meter.clone(),
            out_meter: out_meter.clone(),
        }
        .build(cx);

        ResizeHandle::new(cx);

        VStack::new(cx, |cx| {
            Label::new(cx, "Duro Console")
                //.font_family(vec![FamilyOwned::Name(String::from(assets::NOTO_SANS_THIN,))])
                .font_size(24.0);
                //.height(Pixels(30.0));

            HStack::new(cx, |cx| 
            {
                Label::new(cx, "Sat Type");
                Label::new(cx, "Console Type");
            })
            .col_between(Pixels(120.0))
            .top(Pixels(5.0))
            .bottom(Pixels(0.0));
            //.min_width(Pixels(560.0));

            HStack::new(cx, |cx| 
            {
                ParamSlider::new(cx, Data::params, |params| &params.sat_type);
                ParamSlider::new(cx, Data::params, |params| &params.console_type);
            })
            .col_between(Pixels(10.0))
            .bottom(Pixels(5.0));

            HStack::new(cx, |cx| 
            {
                ParamSlider::new(cx, Data::params, |params| &params.drive).min_width(Pixels(370.0));
            })
            //.col_between(Pixels(10.0))
            .top(Pixels(5.0))
            .bottom(Pixels(5.0));
                
            HStack::new(cx, |cx| 
            {
                ParamSlider::new(cx, Data::params, |params| &params.free_gain);
                ParamSlider::new(cx, Data::params, |params| &params.threshold);
            })
            .col_between(Pixels(10.0))
            .bottom(Pixels(5.0));

            HStack::new(cx, |cx| 
                {
                    ParamSlider::new(cx, Data::params, |params| &params.output_gain);
                    ParamSlider::new(cx, Data::params, |params| &params.dry_wet);
                })
                .col_between(Pixels(10.0))
                .bottom(Pixels(10.0));

            VStack::new(cx, |cx| 
            {
                Label::new(cx, "In");
                PeakMeter::new(cx,
                    Data::in_meter
                        .map(|in_meter| util::gain_to_db(in_meter.load(Ordering::Relaxed))),
                    Some(Duration::from_millis(600)),
                )
                .min_width(Pixels(360.0));
    
                Label::new(cx, "Out");
                PeakMeter::new(cx,
                    Data::out_meter
                        .map(|out_meter| util::gain_to_db(out_meter.load(Ordering::Relaxed))),
                    Some(Duration::from_millis(600)),
                ).min_width(Pixels(360.0));
            })
            .row_between(Pixels(0.0));
            //.child_space(Auto);
            

        })
        .row_between(Pixels(0.0))
        .child_space(Stretch(1.0));
        //.child_right(Stretch(1.0))
        //.child_bottom(Stretch(1.0));
        //.child_left(Stretch(1.0))
        //.child_right(Stretch(1.0));
    })
}