# TODO before sharing/promoting

## DEMO
* replicate something close to DeepNote using gliss
  * required features:
    * random phase shift amount accross different channels

## DEBUG
* plugin crashes on bend > 24 semitones
  * adjustable `PITCH_BEND_RANGE`

## FUNCTIONALITY
* first note should not have a bend of zero

## INTERNAL CODE
* TODO comments
  * read through
  * triage
  * fix/resolve
* clean up and rename `daw_time`/`ui_time`
* remove `allow(dead_code)`
  * revisit bin + lib crate approach to UI iteration

# TODO Backlog

## UI
* flip keyboard so that low notes are at bottom and high notes at top
* render notes differently based on `note.new_note_on`
* display measure bars on timeline

## DEBUG
* bitwig
  * periodic (sin, step, saw, triangle) pitch bend is off slightly
    * seems like not all the pitch bends which "should" be sent from Gliss are received by bitwig
  * non-periodic (s-curve, linear) work as expected
* at on note events, one low pitch bend event is sent which should not be there
* saw bend weird behavior when period between 5 and 6

## FEATURE
* bezier curve bend path
* `n_channels` setting:
  * `= n_notes`
  * `= max (16)`
    * would avoid playing new notes
* presets
* set `bend_duration` based on when keys are released?

## TESTS
* plan out how to reproduce and add test cases for user feedback
* for `get_mapping` variations

## POLISH
* investigate ways to mitigate beating
* parameters
  * chord capture time

## DECISIONS
* do we want new notes durring a bend to have Path::Linear or to also bend?
  * only relevant for Path::
    * sin
    * saw
    * triangle

## LICENCE
* figure out what licence to publish under
