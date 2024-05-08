module ImageSizes exposing (..)

import dict 

type alias Resolution = 
{
    width : Int
,   height : Int
}

type alias ImageSizing = 
{
    original : Resolution 
,   large : Resolution 
,   medium : Resolution 
,   small : Resolution
}

type alias SizeMap = Dict String ImageSizing

imageSizes : SizeMap 
imageSizes = 
 Dict.fromList [("god", { original = { width = 2912, height = 2047 }, large = { width = 1747, height = 1228 }, medium = { width = 873, height = 614 }, small = { width = 436, height = 307 } })]
