
export interface ModelOption {
    input_type: string,
    name: string,
    label: string,
    choices: string[] | null,
    style: string[] | null,
    min: number | null,
    max: number | null,
    value: number | null,
    scale: number | null,
}

export interface Model {
    name: string,
    icon: string,
    model_path: string,
    input_files: number,
    input_bands: number,
    input_ranges: number[],
    post_type: string,
    description: string,
    tags: string[],
    options: ModelOption[],
    license_server: string | null,
}

export interface ModelConfig {
    license_server: string,
    models: Model[],
}

export interface Window {
    col_off: number,
    row_off: number,
    width: number,
    height: number,
}

export interface Geotransform {
    geo: number[],
}

export type RgbaComponnets = [number, number, number, number];
export type HexString = string;
export type Colour = RgbaComponnets | HexString;

export type DiscreteColourDefinition = [number, Colour][];
export type EqualSpaceColourDefinition = Colour[];
export type CustomBreaksColourDefinition = [number, Colour] [];
export type RGBCompositeColourDefinition = [[number, number, number], [number, number, number]];

export type ColourDefinition =  DiscreteColourDefinition | EqualSpaceColourDefinition | CustomBreaksColourDefinition | RGBCompositeColourDefinition;

export interface Style {
    name: string | null,
    colours: ColourDefinition | null,
    vmin: number | null,
    vmax: number | null,
    bands: number[] | null,
}

export interface SpatialInfo {
    epsg_code: number | null,
    proj4: string | null,
    wkt: string | null,
    esri: string | null,
}

export interface MapSettings {
    extent: Window | null,
    path: string,
    name: string,
    geotransform: Geotransform | null,
    no_data_value: number[] | null,
    style: Style | null,
    xml: string | null,
    driver_name: string | null,
    spatial_info: SpatialInfo | null,
    spatial_units: string | null,
    bounds: number[] | null,
    show: boolean,
}