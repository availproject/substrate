use crate::mock::*;
use frame_support::assert_ok;

#[test]
fn it_works_for_default_value() {
    new_test_ext().execute_with(|| {
        // Dispatch a signed extrinsic.
        assert_ok!(TemplateModule::submit_data(
            Origin::signed(1),
            b"key".to_vec(),
            b"value".to_vec()
        ));
    });
}
