use utf8;
use Language::Learner::Test;

use Mojo::UserAgent;
use Language::Learner::GoogleTranslate;

my $gt = Language::Learner::GoogleTranslate->new;

skip_all('Switching off this anyways');

is $gt->spell_check('ceu'), 'céu',
  'got the corrections';

is $gt->spell_check('ceu', 'mae'), 
  { ceu => 'céu',
    mae => 'mãe' },
  'does multi correct also';

is $gt->spell_check('nos'), 'nos',
  'returns correctly spelled st00f';

is $gt->spell_check('nos', 'ceu'),
  { ceu => 'céu',
    nos => 'nos' },
  'mix list fully returns';



done_testing;
