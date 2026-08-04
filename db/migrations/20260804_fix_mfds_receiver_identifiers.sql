WITH route(receiver_label, old_batch, old_message, official_identifier) AS (
    VALUES
        ('MFDS(CT)', 'MFDS_CT', 'CT', 'MFDS-O-CT'),
        ('MFDS(CU)', 'MFDS_CU', 'CU', 'MFDS-O-CU'),
        ('MFDS(KR)', 'MFDS', 'KR', 'MFDS-O-KR'),
        ('MFDS(FR)', 'MFDS_FR', 'FR', 'MFDS-O-FR'),
        ('MFDS(CF)', 'MFDS_CF', 'CF', 'MFDS-O-CF'),
        ('MFDS(CF)', 'MFDS_CT', 'CT', 'MFDS-O-CF')
)
UPDATE submission_receiver_options target
SET batch_receiver_identifier = route.official_identifier,
    message_receiver_identifier = route.official_identifier,
    updated_at = NOW()
FROM route
WHERE target.authority = 'mfds'
  AND target.receiver_label = route.receiver_label
  AND target.batch_receiver_identifier = route.old_batch
  AND target.message_receiver_identifier = route.old_message;

WITH route(receiver_label, old_batch, old_message, official_identifier) AS (
    VALUES
        ('MFDS(CT)', 'MFDS_CT', 'CT', 'MFDS-O-CT'),
        ('MFDS(CU)', 'MFDS_CU', 'CU', 'MFDS-O-CU'),
        ('MFDS(KR)', 'MFDS', 'KR', 'MFDS-O-KR'),
        ('MFDS(FR)', 'MFDS_FR', 'FR', 'MFDS-O-FR'),
        ('MFDS(CF)', 'MFDS_CF', 'CF', 'MFDS-O-CF'),
        ('MFDS(CF)', 'MFDS_CT', 'CT', 'MFDS-O-CF')
)
UPDATE receiver_presave_routes target
SET batch_receiver_identifier = route.official_identifier,
    message_receiver_identifier = route.official_identifier,
    updated_at = NOW()
FROM route
WHERE target.authority = 'mfds'
  AND target.receiver_label = route.receiver_label
  AND target.batch_receiver_identifier = route.old_batch
  AND target.message_receiver_identifier = route.old_message;
