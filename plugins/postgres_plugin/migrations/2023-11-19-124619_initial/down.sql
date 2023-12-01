-- This file should undo anything in `up.sql`

DROP TABLE IF EXISTS alert_alert_annotations;

DROP TABLE IF EXISTS alert_annotation;

DROP TABLE IF EXISTS alert_alert_labels;

DROP TABLE IF EXISTS alert_label;

DROP TABLE IF EXISTS alert;

DROP TABLE IF EXISTS alert_group_common_annotations;

DROP TABLE IF EXISTS common_annotation;

DROP TABLE IF EXISTS alert_group_common_labels;

DROP TABLE IF EXISTS common_label;

DROP TABLE IF EXISTS alert_group_group_labels;

DROP TABLE IF EXISTS group_label;

DROP TABLE IF EXISTS alert_group;

DROP TYPE IF EXISTS alert_status;