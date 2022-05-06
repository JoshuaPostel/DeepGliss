# TODO before sharing/promoting

## DEBUG
* draw notes being bent move poorly when bend duration moved

## DEMO
* replicate something close to DeepNote using gliss

## INTERNAL CODE
* TODO comments
  * read through
  * triage
  * fix/resolve
* clean up and rename `daw_time`/`ui_time`
* remove `allow(dead_code)`
  * revisit bin + lib crate approach to UI iteration

## POLISH
* investigate chord bend on note press that causes popping
* improve default values 

## LICENCE
* figure out what licence to publish under

# TODO Backlog

## FEATURES
* bezier curve bend path
  * requires egui >= 0.17

## UI
* display measure bars on timeline

## DEBUG
* bitwig
  * periodic (sin, step, saw, triangle) pitch bend is off slightly
    * seems like not all the pitch bends which "should" be sent from Gliss are received by bitwig
  * non-periodic (s-curve, linear) work as expected
* at on note events, one low pitch bend event is sent which should not be there
* saw bend weird behavior when period between 5 and 6

## FEATURES
* `n_channels` setting:
  * `= n_notes`
  * `= max (16)`
    * would avoid playing new notes
* set `bend_duration` based on when keys are released?

## TESTS
* plan out how to reproduce and add test cases for user feedback

## POLISH
* investigate ways to mitigate beating
