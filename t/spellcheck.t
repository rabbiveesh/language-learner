use utf8;
use Language::Learner::Test;
use Language::Learner::SpellCheck;


my $s = Language::Learner::SpellCheck->new;

is $s->check('ceu'), 'céu',
  'got the corrections';

is $s->check('ceu', 'mae'), 
  { ceu => 'céu',
    mae => 'mãe' },
  'does multi correct also';

is $s->check('nos'), 'nos',
  'returns correctly spelled st00f';

is $s->check('nos', 'ceu'),
  { ceu => 'céu',
    nos => 'nos' },
  'mix list fully returns';

is $s->check('voce toma agua'),
  'você toma aguá',
  'corrects phrases too';

done_testing;
