import React, { useEffect, useState } from "react";
import { Card, Row, Col, Container, OverlayTrigger, Tooltip } from "react-bootstrap";
import { useSelector } from "react-redux";
import RawJsonButton from "#/RawJsonButton";
import ProtocolParameters from "#/ProtocolParameters";
import Stake from "#/Stake";
import { selectedAggregator } from "@/store/settingsSlice";
import { checkUrl, formatStake } from "@/utils";

function InfoGroupCard({ children, title, ...props }) {
  return (
    <Col xs={12} sm={6} md={4} lg={3} xl={2} className="mb-3" {...props}>
      <Card className="h-100" {...props}>
        <Card.Body>
          <h5>{title}</h5>
          <div className="ps-1">{children}</div>
        </Card.Body>
      </Card>
    </Col>
  );
}

function InfoRow({ label, children, ...props }) {
  return (
    <>
      <div className="d-flex justify-content-between" {...props}>
        <div className="me-2 flex-fill">
          <em>{label}:</em>
        </div>
        <div className="text-end">{children}</div>
      </div>
      <hr className="my-2" />
    </>
  );
}

function PercentTooltip({ value, total, ...props }) {
  return (
    <OverlayTrigger overlay={<Tooltip>{`${value} out of ${total}`}</Tooltip>}>
      <span {...props}>
        <i className="bi bi-question-circle text-secondary"></i>
      </span>
    </OverlayTrigger>
  );
}

export default function AggregatorStatus() {
  const [aggregatorStatus, setAggregatorStatus] = useState({});
  const [aggregatorVersion, setAggregatorVersion] = useState({});
  const currentAggregator = useSelector((state) => state.settings.selectedAggregator);
  const aggregatorStatusEndpoint = useSelector((state) => `${selectedAggregator(state)}/status`);
  const autoUpdate = useSelector((state) => state.settings.autoUpdate);
  const updateInterval = useSelector((state) => state.settings.updateInterval);

  useEffect(() => {
    let fetchAggregatorStatus = () => {
      fetch(aggregatorStatusEndpoint)
        .then((response) => (response.status === 200 ? response.json() : {}))
        .then((data) => setAggregatorStatus(data))
        .catch((error) => {
          setAggregatorStatus({});
          console.error("Fetch status error:", error);
        });
    };

    // Fetch it once without waiting
    fetchAggregatorStatus();

    if (autoUpdate) {
      const interval = setInterval(() => {
        fetchAggregatorStatus();
      }, updateInterval);
      return () => clearInterval(interval);
    }
  }, [aggregatorStatusEndpoint, updateInterval, autoUpdate]);

  useEffect(() => {
    if (!checkUrl(currentAggregator)) {
      return;
    }

    const split_version = aggregatorStatus?.aggregator_node_version?.split("+") ?? [];
    setAggregatorVersion({
      number: split_version[0] ?? "0.0.0",
      sha: split_version[1] ?? undefined,
    });
  }, [currentAggregator, aggregatorStatus]);

  // Calculate percentage without decimal
  function percent(value, total) {
    return ((value / total) * 100).toFixed(0);
  }

  function capitalizeFirstLetter(string) {
    return string.charAt(0).toUpperCase() + string.slice(1);
  }

  return (
    <Container fluid>
      <div>
        <h2>
          Status
          <RawJsonButton href={aggregatorStatusEndpoint} variant="outline-light" size="sm" />
        </h2>

        <Container fluid>
          <Row className="d-flex flex-wrap justify-content-md-center">
            <InfoGroupCard title={`Epoch ${aggregatorStatus.epoch}`}>
              <InfoRow label="Cardano Era">{aggregatorStatus.cardano_era}</InfoRow>
              <InfoRow label="Mithril Era">
                {aggregatorStatus.mithril_era
                  ? capitalizeFirstLetter(aggregatorStatus.mithril_era)
                  : ""}
              </InfoRow>
            </InfoGroupCard>

            <InfoGroupCard title="SPOs">
              <InfoRow label="Current Signers">{aggregatorStatus.total_signers ?? 0}</InfoRow>
              <InfoRow label="Next Signers">{aggregatorStatus.total_next_signers ?? 0}</InfoRow>
              <InfoRow label="Cardano Adoption">
                {percent(aggregatorStatus.total_signers, aggregatorStatus.total_cardano_spo)}%{" "}
                <PercentTooltip
                  value={aggregatorStatus.total_signers ?? 0}
                  total={aggregatorStatus.total_cardano_spo ?? 0}
                />
              </InfoRow>
            </InfoGroupCard>

            <InfoGroupCard title="Stakes">
              <InfoRow label="Current Signers">
                <Stake lovelace={aggregatorStatus.total_stakes_signers ?? 0} />
              </InfoRow>
              <InfoRow label="Next Signers">
                <Stake lovelace={aggregatorStatus.total_next_stakes_signers ?? 0} />
              </InfoRow>
              <InfoRow label="Cardano Adoption">
                {percent(
                  aggregatorStatus.total_stakes_signers,
                  aggregatorStatus.total_cardano_stake,
                )}
                %{" "}
                {/* <IconBadge
                  tooltip={`${formatStake(aggregatorStatus.total_stakes_signers) ?? 0} out of ${formatStake(aggregatorStatus.total_cardano_stake) ?? 0}`}
                  icon="question-circle"
                  className="p-0"
                  variant="body-secondary"
                /> */}
                <PercentTooltip
                  value={formatStake(aggregatorStatus.total_stakes_signers) ?? 0}
                  total={formatStake(aggregatorStatus.total_cardano_stake) ?? 0}
                />
              </InfoRow>
            </InfoGroupCard>

            <InfoGroupCard title="Protocol Parameters">
              <em>Current:</em>
              <ProtocolParameters protocolParameters={aggregatorStatus.protocol} padding={2} />
              <em>Next:</em>
              <ProtocolParameters protocolParameters={aggregatorStatus.next_protocol} padding={2} />
            </InfoGroupCard>

            <InfoGroupCard title="Versions">
              <InfoRow label="Aggregator">
                {aggregatorVersion.number}
                {aggregatorVersion.sha && <em> ({aggregatorVersion.sha})</em>}
              </InfoRow>
              <InfoRow label="Cardano">{aggregatorStatus.cardano_node_version}</InfoRow>
            </InfoGroupCard>
          </Row>
        </Container>
      </div>
    </Container>
  );
}
