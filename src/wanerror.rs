use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WanError {
    #[error("an input/output error happened")]
    IOError(#[from] io::Error),
    #[error("an image id reference the previous one, but it is the first image")]
    ImageIDPointBackButFirstImage,
    #[error("a metaframe is inferior to -1, but that is not valid (it is {0})")]
    MetaFrameLessThanLessOne(i16),
    #[error("While creating a meta frame store: the check for the offset of the pointer of the animation group are not valid!")]
    InvalidOffset,
    #[error("the resolution for an image wasn't found")]
    InvalidResolution,
    #[error("pointer to image parts are not coherent")]
    IncoherentPointerToImagePart,
    #[error("impossible to find a definied z_index (the image is probably empty)")]
    NoZIndex,
    #[error("an invalid alpha level was found in the picture")]
    ImpossibleAlphaLevel,
    #[error("an image data pointer is null")]
    NullImagePointer,
    #[error("the image does not have a resolution")]
    ImageWithoutResolution,
    #[error("the palette data doesn't end with 0s")]
    PaletteDontEndWithZero,
    #[error("a reference to a color in a palette would overflow")]
    PaletteOOB,
    #[error("can't find a specific color in the palette")]
    CantFindColorInPalette,
    #[error("the sir0 header in invalid, expected SIR0, found {0:?}")]
    InvalidSir0([u8; 4]),
    #[error("the end of the sir0 header should be four 0, found {0:?}")]
    InvalidEndOfSir0Header([u8; 4]),
    #[error("the type of sprite is unknown (found the sprite type id {0}, but this program only known sprite for [0, 1, 3])")]
    TypeOfSpriteUnknown(u16),
    #[error("the 2 byte that indicate the number of color is invalid (found {0}, expected 0 or 1")]
    InvalidColorNumber(u16),
    #[error("the value of a substraction is less than 0: {0}-{1} ({2}-{3})")]
    OverflowSubstraction(u64, u64, &'static str, &'static str),
    #[error("the value of an addition is more than the maximum possible value: {0}+{1} ({2}+{3})")]
    OverflowAddition(u64, u64, &'static str, &'static str),
    #[error("the resolution of a sprite is too small accept all it's pixel")]
    SpriteTooSmall,
    #[error("an image doesn't have a constant depth index")]
    NonConstantIndexInImage,
    #[error("The pointer to {0} is reference content after the end of the file")]
    PostFilePointer(&'static str),
}
