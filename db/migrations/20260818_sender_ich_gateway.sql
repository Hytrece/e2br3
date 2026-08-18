ALTER TABLE sender_presave_gateways
    DROP CONSTRAINT sender_presave_gateways_authority_valid,
    ADD CONSTRAINT sender_presave_gateways_authority_valid
        CHECK (gateway_authority IN ('ich', 'fda', 'mfds'));
