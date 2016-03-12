module Main where

import Control.Exception (bracket)
import Graphics.X11 (openDisplay, closeDisplay, defaultScreenOfDisplay)
import Graphics.X11.Xft (xftFontOpen, xftFontClose, xftfont_height)
import System.Environment (getArgs)

main :: IO ()
main = do
    desc:_ <- getArgs
    bracket (openDisplay "") closeDisplay $ \dpy ->
        bracket (xftFontOpen dpy (defaultScreenOfDisplay dpy) desc) (xftFontClose dpy) $ \font ->
            xftfont_height font >>= print
