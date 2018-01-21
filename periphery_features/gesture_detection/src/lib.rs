extern crate periphery_core;

extern crate signals;


use periphery_core::*;
use periphery_core::prelude::v1::*;

use self::signals::smoothed_z_score::*;

#[cfg(test)]
extern crate csv;
#[cfg(test)]
extern crate gnuplot;

#[cfg(test)]
#[macro_use]
extern crate serde_derive;


#[derive(Copy, Clone, Debug)]
pub struct GestureSensorPoint {
    pub milliseconds: f32,
    pub up: f32,
    pub down: f32,
    pub left: f32,
    pub right: f32
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum GestureSensorAxis {
    UpDown,
    LeftRight
}

#[derive(Copy, Clone, Debug)]
pub struct GesturePeak {
    id: usize,
    axis: GestureSensorAxis,
    peak: Peak
}

#[derive(Default)]
pub struct GestureDetector {
    peaks: Vec<GesturePeak>,
    up_down: Vec<f32>,
    left_right: Vec<f32>,
    
    avg_samples: usize
}

impl GestureDetector {
    pub fn new() -> Self {
        let mut g: Self = Default::default();
        g.avg_samples = 5;
        g
    }

    fn mean_raw_data(&self, data: &[GestureSensorPoint]) -> Vec<GestureSensorPoint> {
        let up = mean_filter(&data.iter().map(|&p| p.up).collect::<Vec<f32>>(), self.avg_samples);
        let down = mean_filter(&data.iter().map(|&p| p.down).collect::<Vec<f32>>(), self.avg_samples);
        let left = mean_filter(&data.iter().map(|&p| p.left).collect::<Vec<f32>>(), self.avg_samples);
        let right = mean_filter(&data.iter().map(|&p| p.right).collect::<Vec<f32>>(), self.avg_samples);

        let mut ret = Vec::with_capacity(data.len());

        for i in 0..up.len() {
            let p = GestureSensorPoint {
                milliseconds: data[i].milliseconds,
                up: up[i],
                down: down[i],
                left: left[i],
                right: right[i]
            };
            ret.push(p);
        }

        ret
    }

    pub fn detect(&mut self, data: &[GestureSensorPoint]) -> Vec<Gesture> {        
        let mut ret = vec![];       
        if data.len() == 0 { return ret; }

        let avg_samples = self.avg_samples;

        let data_mean = self.mean_raw_data(data);
        
        let max_abs_value = {
            let up = iter_partial_max(data_mean.iter().map(|p| p.up.abs())).unwrap();
            let down = iter_partial_max(data_mean.iter().map(|p| p.down.abs())).unwrap();
            let left = iter_partial_max(data_mean.iter().map(|p| p.left.abs())).unwrap();
            let right = iter_partial_max(data_mean.iter().map(|p| p.right.abs())).unwrap();

            let values = [up, down, left, right];
            iter_into_partial_max(values.iter()).cloned().unwrap()
        };
        
        let noise_threshold = max_abs_value * 0.075;
        if noise_threshold < 1.0 { return ret; }


        let diff_axis = |a: Vec<f32>, b: Vec<f32>| {
            let a = mean_filter(&a, avg_samples);
            let b = mean_filter(&b, avg_samples);

            let mut r = vec![];
            for (a, b) in a.iter().zip(b.iter()) {
                let noisiness = 1.0 - ((a + b) / (noise_threshold * 2.0)).max(1.0);

                let v = ((a - b) / (a + b)) * noisiness;
                r.push(v);
            }

            r
        };

        self.up_down = diff_axis(data.iter().map(|p| p.up).collect(), data.iter().map(|p| p.down).collect());
        self.left_right = diff_axis(data.iter().map(|p| p.left).collect(), data.iter().map(|p| p.right).collect());

        {
            let ud = self.find_peaks(&self.up_down);
            let lr = self.find_peaks(&self.left_right);

            self.peaks.extend(ud.iter().map(|p| GesturePeak { peak: *p, axis: GestureSensorAxis::UpDown, id: 0 }));
            self.peaks.extend(lr.iter().map(|p| GesturePeak { peak: *p, axis: GestureSensorAxis::LeftRight, id: 0 }));
        }

        self.peaks.sort_by_key(|p| p.peak.start.idx);

        {
            let mut id = 1;
            for ref mut peak in &mut self.peaks {
                peak.id = id;
                id += 1;
            }
        }

        let lr_peaks_iter = || { self.peaks.iter().filter(|&p| p.axis == GestureSensorAxis::LeftRight) };
        let ud_peaks_iter = || { self.peaks.iter().filter(|&p| p.axis == GestureSensorAxis::UpDown) };

        let mut used_peak_ids = vec![];
        
        let max_gesture_duration = 250;

        for peak in &self.peaks {
            if used_peak_ids.contains(&peak.id) { continue; }

            let next_peaks: Vec<_> = self.peaks.iter().filter(|p| {
                let d = (p.peak.start.idx as isize) - (peak.peak.start.idx as isize);
                d >= 0 && d <= max_gesture_duration
            }).collect();
            
            let mut i = next_peaks.len() as isize;
            loop {
                let potential_peaks = &next_peaks[0..i as usize];
                let threshold = 0.25;
                

                {
                    trait FindGesture {
                        fn find_gesture(matcher: &mut PeakMatcher) -> Result<Gesture, PeakMatcherError>;
                    }

                    struct SwipeRight;
                    impl FindGesture for SwipeRight {
                        fn find_gesture(matcher: &mut PeakMatcher) -> Result<Gesture, PeakMatcherError> {
                            let lr_peak = matcher.saturated_peak_positive_first(GestureSensorAxis::LeftRight)?;
                            let diff = PeakDiffs::diff(lr_peak.0.peak, lr_peak.1.peak);

                            let gesture = Gesture {
                                kind: GestureKind::SwipeToRight,
                                pos: diff.start,
                                len: diff.len
                            };

                            Ok(gesture)
                        }
                    }

                    struct SwipeDown;
                    impl FindGesture for SwipeDown {
                        fn find_gesture(matcher: &mut PeakMatcher) -> Result<Gesture, PeakMatcherError> {
                            let ud_peak = matcher.saturated_peak_positive_first(GestureSensorAxis::UpDown)?;
                            let diff = PeakDiffs::diff(ud_peak.0.peak, ud_peak.1.peak);

                            let lr_positive_peaks = matcher.saturated_peaks(1.0, GestureSensorAxis::LeftRight)?;

                            let gesture = Gesture {
                                kind: GestureKind::SwipeToDown,
                                pos: diff.start,
                                len: diff.len
                            };

                            Ok(gesture)
                        }
                    }

                    struct SwipeLeft;
                    impl FindGesture for SwipeLeft {
                        fn find_gesture(matcher: &mut PeakMatcher) -> Result<Gesture, PeakMatcherError> {
                            let lr_peak = matcher.saturated_peak_negative_first(GestureSensorAxis::LeftRight)?;
                            let diff = PeakDiffs::diff(lr_peak.0.peak, lr_peak.1.peak);

                            let gesture = Gesture {
                                kind: GestureKind::SwipeToLeft,
                                pos: diff.start,
                                len: diff.len
                            };

                            Ok(gesture)
                        }
                    }

                    struct SwipeUp;
                    impl FindGesture for SwipeUp {
                        fn find_gesture(matcher: &mut PeakMatcher) -> Result<Gesture, PeakMatcherError> {
                            let ud_peak = matcher.saturated_peak_negative_first(GestureSensorAxis::UpDown)?;
                            let diff = PeakDiffs::diff(ud_peak.0.peak, ud_peak.1.peak);

                            let lr_positive_peaks = matcher.saturated_peaks(1.0, GestureSensorAxis::LeftRight)?;

                            let gesture = Gesture {
                                kind: GestureKind::SwipeToUp,
                                pos: diff.start,
                                len: diff.len
                            };

                            Ok(gesture)
                        }
                    }

                    struct HandNear;
                    impl FindGesture for HandNear {
                        fn find_gesture(matcher: &mut PeakMatcher) -> Result<Gesture, PeakMatcherError> {
                            let min_saturated_points = 150;
                            let min_saturated_value = 200.0;

                            // on LRUD sensor data
                            let any_peak = matcher.peak_from_start(|_| { true })?;

                            //let mut idx = any_peak.peak.end.idx;
                            //println!("idx = {}", idx);
                            let mut n = 0;

                            // todo: out of bounds!!!
                            for p in &matcher.sensor_data[any_peak.peak.end.idx..] {

                                //println!("p = {:?}", p);

                                if p.up > min_saturated_value &&
                                   p.down > min_saturated_value &&
                                   p.left > min_saturated_value &&
                                   p.right > min_saturated_value
                                {
                                    //idx += 1;
                                    n += 1;
                                } else {
                                    break;
                                }                                
                            }

                            if n > min_saturated_points {
                                //println!("n = {}", n);

                                let range = any_peak.peak.start.idx..(any_peak.peak.end.idx + n);

                                for &peak in matcher.all_peaks {
                                    if range.start < peak.peak.start.idx && peak.peak.start.idx < range.end {
                                        if !matcher.matched_peak_ids.contains(&peak.id) {
                                            matcher.matched_peak_ids.push(peak.id);
                                        }
                                    }
                                }


                                let gesture = Gesture {
                                    kind: GestureKind::HandNear,
                                    pos: any_peak.peak.end.idx,
                                    len: n
                                };

                                Ok(gesture)

                            } else {                                                        
                                //let start = any_peak.peak.start.idx;                            
                                //println!("data: {:?}", matcher.sensor_data[start]);

                                Err(PeakMatcherError::NoMatch)
                            }
                        }
                    }

                    fn find_gestures(matcher: PeakMatcher) -> Result<(Gesture, PeakMatcher), PeakMatcherError> {

                        fn run<F: FindGesture>(mut matcher: PeakMatcher) -> Result<(Gesture, PeakMatcher), PeakMatcherError> {
                            F::find_gesture(&mut matcher).map(|g| (g, matcher))
                        }

                        run::<SwipeRight>(matcher.clone())
                            .or_else(|_| { run::<SwipeLeft>(matcher.clone()) })
                            .or_else(|_| { run::<SwipeUp>(matcher.clone()) })
                            .or_else(|_| { run::<SwipeDown>(matcher.clone()) })
                            .or_else(|_| { run::<HandNear>(matcher.clone()) })
                    }

                    let mut matcher = PeakMatcher::new(potential_peaks, &data_mean);
                    if let Ok((g, matcher)) = find_gestures(matcher) {
                        ret.push(g);
                        for id in matcher.matched_peak_ids {
                            used_peak_ids.push(id);
                        }
                        break;
                    }
                }




                i -= 1;
                if i == 0 { break; }
            }
        }

        ret
    }

    fn find_peaks(&self, y: &[f32]) -> Vec<Peak> {
        let settings = ZScoreSettings {
            lag: 1,
            threshold: 2.5,
            influence: 0.6,
            min_abs_peak_value: 0.05
        };

        find_peaks(y, settings)
    }
}


#[derive(Copy, Clone, Debug)]
pub struct PeakDiffs {
    pub start: usize,
    pub len: usize
}

impl PeakDiffs {
    pub fn diff(a: Peak, b: Peak) -> Self {
        PeakDiffs {
            start: a.start.idx,
            len: b.end.idx - a.start.idx
        }
    }
}

#[derive(Clone, Debug)]
pub struct PeakMatcher<'a> {
    sensor_data: &'a [GestureSensorPoint],
    all_peaks: &'a [&'a GesturePeak],
    matched_peak_ids: Vec<usize>,
    position_id: Option<usize>
}

#[derive(Copy, Clone, Debug)]
pub enum PeakMatcherError {
    NoMatch
}

impl<'a> PeakMatcher<'a> {
    fn new(peaks: &'a [&'a GesturePeak], sensor_data: &'a [GestureSensorPoint]) -> Self {
        PeakMatcher {
            sensor_data: sensor_data,
            all_peaks: peaks,
            matched_peak_ids: vec![],
            position_id: None
        }
    }

    fn is_already_matched(&self, peak: &GesturePeak) -> bool {
        self.matched_peak_ids.contains(&peak.id)
    }

    fn peak_from_start<F: Fn(&GesturePeak) -> bool>(&mut self, f: F) -> Result<&'a GesturePeak, PeakMatcherError> {
        for peak in self.all_peaks {
            if !self.is_already_matched(peak) && f(peak) {
                self.matched_peak_ids.push(peak.id);
                self.position_id = Some(peak.id);
                return Ok(peak);
            }
        }

        Err(PeakMatcherError::NoMatch)
    }

    fn peak<F: Fn(&GesturePeak) -> bool>(&mut self, f: F) -> Result<&'a GesturePeak, PeakMatcherError> {
        if let Some(position_id) = self.position_id {
            for peak in self.all_peaks.iter().filter(|p| p.id > position_id) {
                if !self.is_already_matched(peak) && f(peak) {
                    self.matched_peak_ids.push(peak.id);
                    self.position_id = Some(peak.id);
                    return Ok(peak);
                }
            }
        }

        Err(PeakMatcherError::NoMatch)
    }

    fn saturated_peak_positive_first(&mut self, axis: GestureSensorAxis) -> Result<(&'a GesturePeak, &'a GesturePeak), PeakMatcherError> {
        let threshold = 0.25;

        let p1 = self.peak_from_start(|p| p.axis == axis && p.peak.max_deviation.val > threshold)?;
        let p2 = self.peak(|p| p.axis == axis && p.peak.max_deviation.val < -threshold)?;

        Ok((p1, p2))
    }

    fn saturated_peak_negative_first(&mut self, axis: GestureSensorAxis) -> Result<(&'a GesturePeak, &'a GesturePeak), PeakMatcherError> {
        let threshold = 0.25;

        let p1 = self.peak_from_start(|p| p.axis == axis && p.peak.max_deviation.val < -threshold)?;
        let p2 = self.peak(|p| p.axis == axis && p.peak.max_deviation.val > threshold)?;

        Ok((p1, p2))
    }

    fn saturated_peaks(&mut self, factor: f32, axis: GestureSensorAxis) -> Result<(&'a GesturePeak, &'a GesturePeak), PeakMatcherError> {
        let threshold = 0.25 * factor;

        let p1 = self.peak_from_start(|p| p.axis == axis && p.peak.max_deviation.val > threshold)?;
        let p2 = self.peak(|p| p.axis == axis && p.peak.max_deviation.val > threshold)?;

        Ok((p1, p2))
    }

    
}


#[derive(Copy, Clone, Debug, PartialEq)]
pub enum GestureKind {
    SwipeToRight,
    SwipeToLeft,
    SwipeToDown,
    SwipeToUp,
    HandNear
}

impl GestureKind {
    pub fn flip_upside_down(&self) -> Self {
        match *self {
            GestureKind::SwipeToRight => GestureKind::SwipeToLeft,
            GestureKind::SwipeToLeft => GestureKind::SwipeToRight,
            GestureKind::SwipeToDown => GestureKind::SwipeToUp,
            GestureKind::SwipeToUp => GestureKind::SwipeToDown,
            GestureKind::HandNear => GestureKind::HandNear
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Gesture {
    pub kind: GestureKind,
    pub pos: usize,
    pub len: usize
}

impl Gesture {
    pub fn flip_upside_down(&self) -> Self {
        let mut g = *self;
        g.kind = g.kind.flip_upside_down();
        g
    }
}


fn mean(v: &[f32]) -> f32 {
    if v.len() == 0 { return 0.0; }
    let mut mean = 0.0;
    for &n in v {
        mean += n;
    }
    mean / v.len() as f32
}

/// Delta Degrees of Freedom
#[derive(Clone, Copy, PartialEq)]
enum Population {
    /// DDOF = 0
    Finite,
    /// DDOF = 1
    Entire    
}

impl Default for Population {
    fn default() -> Self {
        Population::Finite
    }
}

fn std(v: &[f32], population: Population) -> f32 {
    if v.len() == 0 { return 0.0; }

    let mean = mean(v);
    let mut s = 0.0;
    for &n in v {
        let x = n - mean;
        s += x*x;
    }

    let n = v.len() as f32;
    let d = if population == Population::Entire {
        s / (n-1.0)
    } else {
        s / n
    };
    d.sqrt()
}

fn mean_filter(y: &[f32], samples: usize) -> Vec<f32> {
    let mut ret = Vec::with_capacity(y.len());
    for i in 0..y.len() {
        let o: usize = cmp::max(0, i as isize - samples as isize + 1) as usize;
        let m: f32 = y.iter().skip(o).take(samples).sum();
        let m = m / samples as f32;
        ret.push(m);
    }
    ret
}

#[derive(Copy, Clone, Debug)]
pub struct Bin {
    pub from: f32,
    pub to: f32,
    pub n: usize
}

fn histogram(y: &[f32], n: usize) -> Vec<Bin> {
    let mut h = vec![];
    let min = f32_min(y).1;
    let max = f32_max(y).1;

    let s = (max - min) / n as f32;

    for i in 0..n {
        let mut bin = Bin {
            from: min + (s * i as f32),
            to: min + (s * (i + 1) as f32),
            n: 0
        };

        for &v in y {
            if v >= bin.from && v < bin.to {
                bin.n += 1;
            }
        }

        h.push(bin);
    }

    h    
}

fn f32_max(y: &[f32]) -> (usize, f32) {
    let mut idx = 0;
    let mut max = -3.40282347e+38f32;    
    for (i, &v) in y.iter().enumerate() {
        if v > max {
            max = v;
            idx = i;
        }
    }
    (idx, max)
}

fn f32_min(y: &[f32]) -> (usize, f32) {
    let mut idx = 0;
    let mut min = 3.40282347e+38f32;
    for (i, &v) in y.iter().enumerate() {
        if v < min {
            min = v;
            idx = i;
        }
    }
    (idx, min)
}


#[derive(Debug, Copy, Clone)]
pub struct PercSlice {
    pub from: usize,
    pub to: usize
}

fn median_perc_slice(y: &[f32], percentage: usize) -> Option<PercSlice> {
    let mut f: Vec<_> = y.into();
    //println!("y: {:?}", y);
    f.sort_by(|a, b| a.partial_cmp(b).unwrap());
    //println!("f: {:?}", f);

    if let Some(&l) = f.get((f.len() * percentage) / 100) {
        //println!("l = {:?}", l);
        
        let (mut from, mut to) = (None, None);

        for (i, &v) in y.iter().enumerate() {
            if from.is_none() {
                if v >= l {
                    from = Some(i);
                }
            } else if to.is_none() {
                if v < l {
                    to = Some(i);
                }
            }
        }

        match (from, to) {
            (Some(from), Some(to)) => {
                return Some(PercSlice {
                    from: from,
                    to: to
                });
            },
            (_, _) => ()
        }        
    } 

    None
}

#[cfg(test)]
#[test]
fn test_csv_gesture_sensor() {

    let data = include_bytes!("apds9960_i2c_0x39_2017-07-25 072728.csv");
    gesture_detector("1", data, None);
    
    let data = include_bytes!("apds9960_i2c_0x39_2017-07-25 100543.csv");
    gesture_detector("2", data, None);

    let data = include_bytes!("apds9960_i2c_0x39_2017-07-26 152026.csv");
    gesture_detector("3_r", data, Some(1000..2300));
    gesture_detector("3", data, None);


    let data = include_bytes!("apds9960_i2c_0x39_2017-09-06 122127.csv");
    gesture_detector("4_down_up", data, None);

    let data = include_bytes!("apds9960_i2c_0x39_2017-09-06 135939.csv");
    gesture_detector("5_button", data, None);

    
    
}

#[cfg(test)]
fn gesture_detector(filename: &str, data: &[u8], subset: Option<Range<usize>>) {

    return;
    
    use self::gnuplot::*;
    //use self::signals::smoothed_z_score::*;

    use std::cmp;
    use std::f32;

    #[derive(Deserialize, Debug)]
    struct Sample {
        num: usize,
        ms: f32,
        up: f32,
        down: f32,
        left: f32,
        right: f32
    }

    println!("=========");
    println!("file: {}", filename);


    let mut rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(&data[..]);
    
    let gesture: Vec<Sample> = rdr.deserialize().map(|r| r.unwrap()).collect();

    let mut p: Vec<_> = gesture.iter().map(|p| GestureSensorPoint {
        milliseconds: p.ms,
        up: p.up,
        down: p.down,
        left: p.left,
        right: p.right
    }).collect();
    if let Some(subset) = subset {
        p = p.into_iter().skip(subset.start).take(subset.end - subset.start).collect();
    }

    let mut detector = GestureDetector::new();
    let gestures = detector.detect(&p);
    println!("detected gestures: {:?}", gestures);
    //println!("peaks: {:?}", detector.peaks);

    let mut fg = Figure::new();

    {
        let mut axes2d = fg.axes2d();
        axes2d.set_border(true, &[Left, Bottom], &[LineWidth(2.0)]);

        
        let x: Vec<_> = (0..p.len()).collect();

        {
            let mut plot_sensor_data = |y: &[f32], caption: &str, color: &str| {
                let y = mean_filter(y, detector.avg_samples);
                axes2d.lines(&x, &y, &[Caption(caption), LineWidth(1.5), Color(color)]);
            };
            
            /*
            plot_sensor_data(&p.iter().map(|p| p.up).collect::<Vec<_>>(), "Up", "black");
            plot_sensor_data(&p.iter().map(|p| p.down).collect::<Vec<_>>(), "Down", "red");
            plot_sensor_data(&p.iter().map(|p| p.left).collect::<Vec<_>>(), "Left", "green");
            plot_sensor_data(&p.iter().map(|p| p.right).collect::<Vec<_>>(), "Right", "blue");
            */
        }

        {
            let mut plot_filtered_axis_data = |y: &[f32], caption: &str, color: &str| {
                axes2d.lines(&x, y, &[Caption(caption), LineWidth(1.5), Color(color)]);
            };

            plot_filtered_axis_data(&detector.up_down, "Up/Down", "red");
            plot_filtered_axis_data(&detector.left_right, "Left/Right", "green");
        }

        {
            {
                let mut draw_peaks = |axis: GestureSensorAxis, caption: &str, color: &str| {
                    let mut px = vec![];
                    let mut py = vec![];
                    for peak in detector.peaks.iter().filter(|p| p.axis == axis) {
                        let peak = peak.peak;
                        px.push(peak.max_deviation.idx as f32);
                        py.push(peak.max_deviation.val as f32);
                    }

                    axes2d.points(&px, &py, &[Caption(caption), Color(color), PointSize(1.0), PointSymbol('O')]);
                };

                
                draw_peaks(GestureSensorAxis::UpDown, "Up/Down Peaks", "red");
                draw_peaks(GestureSensorAxis::LeftRight, "Left/Right Peaks", "green");    
            }


            {
                let s = [ArrowSize(0.05), ArrowType(ArrowheadType::NoArrow), LineWidth(10.0)];

                for peak in &detector.peaks {
                    let peak = peak.peak;

                    axes2d.arrow(
                        Coordinate::Axis(peak.start.idx as f64),
                        Coordinate::Axis(peak.start.val as f64),
                        Coordinate::Axis(peak.max_deviation.idx as f64),
                        Coordinate::Axis(peak.max_deviation.val as f64),
                        &s
                    );

                    axes2d.arrow(
                        Coordinate::Axis(peak.max_deviation.idx as f64),
                        Coordinate::Axis(peak.max_deviation.val as f64),
                        Coordinate::Axis(peak.end.idx as f64),
                        Coordinate::Axis(peak.end.val as f64),                    
                        &s
                    );
                }                
            }
        }

        {
            let mut label_offset = 0.0;
            let label_offset_rep = 1.0;
            let label_offset_step = 0.10;
            
            let mut gestures = gestures.clone();
            gestures.sort_by_key(|s| s.pos);
            for gesture in &gestures {
                let y = -2.0 - label_offset;
                label_offset = (label_offset + label_offset_step) % label_offset_rep;

                axes2d.label(
                    &format!("{:?}", gesture.kind),
                    Coordinate::Axis(gesture.pos as f64),
                    Coordinate::Axis(y),
                    &[TextAlign(AlignType::AlignCenter)]
                );
            }

            let mut px = vec![];
            let mut py = vec![];

            for gesture in &gestures {
                px.push((gesture.pos + (gesture.len / 2)) as f32);
                py.push(0.0);
            }

            axes2d.points(&px, &py, &[Caption("Step midpoints"), Color("purple"), PointSize(2.0), PointSymbol('*')]);
        }
    }

    fg.set_terminal("pngcairo size 2500, 1600", &format!("plot_{}.png", filename));
	fg.show();

    println!("=========");
    
}