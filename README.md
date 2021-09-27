# Bottlerocket Update Operator v0.2.0

## Notice

This branch of the Bottlerocket Update Operator is an in-development redesign of the project.
Please see the [`develop` branch](https://github.com/bottlerocket-os/bottlerocket-update-operator/tree/develop) for instructions on using the current Bottlerocket Update Operator.


The Bottlerocket update operator is a [Kubernetes operator](https://Kubernetes.io/docs/concepts/extend-Kubernetes/operator/) that coordinates Bottlerocket updates on hosts in a cluster.
When installed, the Bottlerocket update operator starts a controller deployment on one node and agent daemon set on every Bottleorocket node, which takes care of periodically querying updates, draining the node, and performing an update when asked by controller.
Updates to Bottlerocket are rolled out in [waves](https://github.com/bottlerocket-os/bottlerocket/tree/develop/sources/updater/waves) to reduce the impact of issues; the nodes in your cluster may not all see updates at the same time.

## Security

See [CONTRIBUTING](CONTRIBUTING.md#security-issue-notifications) for more information.

## License

This project is dual licensed under either the Apache-2.0 License or the MIT license, your choice.