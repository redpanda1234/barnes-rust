#[derive(Debug)] // for printing stuff
pub struct region {
    pub ne: Option<region>; // Option enum; either some or none.
    pub nw: Option<region>; // Makes handling dead regions easier.
    pub sw: Option<region>;
    pub se: Option<region>;

    pub x: f64; // use the bottom-left corner as the reference point
    pub y: f64; //

    pub masses: Option<Vec<body>> // Optionally include the bodies?
}
