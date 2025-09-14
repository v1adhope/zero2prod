create table if not exists subscriptions(
  subscription_id uuid,
  email text,
  name text,
  created_at timestamptz,

  constraint pk_subscriptions_subscription_id primary key(subscription_id),
  constraint unique_subscriptions_email unique(email)
);
