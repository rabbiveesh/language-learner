use Language::Learner::Test;
use Language::Learner::GoogleImages;
use Future::AsyncAwait;

ok my $g_images_pt = Language::Learner::GoogleImages->new, 
  'can instantiate';

my $images;
async sub get_them {
  $images = await $g_images_pt->get_images(@_)
}


get_them('bolacha')->wait;

my $count = $images->[0]{results}->@*;
is $count, 20, 'got proper count of images';


done_testing;
