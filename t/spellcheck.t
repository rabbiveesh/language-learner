use utf8;
use Language::Learner::Test;
use Language::Learner::SpellCheck;


my $s = Language::Learner::SpellCheck->new;

subtest 'Normal functionality' => sub {
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

  is my $hash = { $s->check('voce toma agua') , 'hi' },
     { 'você toma aguá' => 'hi' },
     'hash keys are normal';

  is [ keys $hash->%* ], [ 'você toma aguá' ],
    'keys returns as expected';
};

subtest 'on filenames' => sub {
  my $voce = curfile->sibling('mp3s')->list->grep(qr/you drink/)->[0];
  $voce = decode 'UTF-8', (split /- /, $voce->basename('.mp3'))[1];
  is $s->check($voce), 'você toma aguá', 'did the phrase right';
};

done_testing;
