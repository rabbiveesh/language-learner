use utf8;
use Language::Learner::Test;

use Mojo::UserAgent;
use Language::Learner::GoogleTranslate;

my $gt = Language::Learner::GoogleTranslate->new;

is $gt->spell_check('ceu'), 'céu',
  'got the corrections';

is $gt->spell_check('ceu', 'mae'), 
  { ceu => 'céu',
    mae => 'mãe' },
  'does multi correct also';




done_testing;
