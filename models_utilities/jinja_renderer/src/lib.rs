#[cfg(test)]
mod tests {
    use minijinja::{context, Environment};
    use random_models_generator::generate_random_alertmanager_pushes;

    #[test]
    fn render() {
        let push = generate_random_alertmanager_pushes(1).pop().unwrap();
        println!("{:#?}", push);
        let mut env = Environment::new();
        env.add_template(
            "push",
            r#"
        Version: {{ push.version }}
        Group key: {{ push.groupKey }}
        Truncated alerts: {{ push.truncatedAlerts }}
        Status: {{ push.status }}
        Receiver: {{ push.receiver }}
        Group labels: 
        {% for key, value in push.groupLabels|items %}
            {{ key }}: {{ value }}
        {% endfor %}
        Common labels: 
        {% for key, value in push.commonLabels|items %}
            {{ key }}: {{ value }}
        {% endfor %}
        Common annotations: 
        {% for key, value in push.commonAnnotations|items %}
            {{ key }}: {{ value }}
        {% endfor %}
        External URL: {{ push.externalURL }}
        Alerts: 
        {% for alert in push.alerts %}
            Status: {{ alert.status }}
            Name: {{ alert.labels.alertname }}
            Lables: 
            {% for key, value in alert.labels|items %}{% if key != "alertname" %}
                {{ key }}: {{ value }}
            {% endif %}{% endfor %}
            Annotations: 
            {% for key, value in alert.annotations|items %}
                {{ key }}: {{ value }}
            {% endfor %}
            Starts at: {{ alert.startsAt }}
            Ends at: {{ alert.endsAt }}
            Generator URL: {{ alert.generatorURL }}
            Fingerprint: {{ alert.fingerprint }}
        {% endfor %}
        "#,
        )
        .unwrap();
        let template = env.get_template("push").unwrap();
        println!("{}", template.render(context! { push => push }).unwrap());
    }
}
