use conrod::input::state::mouse::Button as ConrodButton;
use wlc::Button as WlcButton;

fn wlc_to_conrod_button(button: WlcButton) -> ConrodButton
{
    match button {
        WlcButton::Left => ConrodButton::Left,
        WlcButton::Right => ConrodButton::Right,
        WlcButton::Middle => ConrodButton::Middle,
        WlcButton::Side => ConrodButton::X1,
        WlcButton::Extra => ConrodButton::X2,
        WlcButton::Forward => ConrodButton::Button6,
        WlcButton::Back => ConrodButton::Button7,
        WlcButton::Task => ConrodButton::Button8,
    }
}
