import clsx from "clsx";
import Heading from "@theme/Heading";
import styles from "./styles.module.css";
import { Icon } from "@iconify/react"; // Import the entire Iconify library

type FeatureItem = {
  title: string;
  icon: string;
  description: JSX.Element;
};

const FeatureList: FeatureItem[] = [
  {
    title: "Bring your Own Tools",
    icon: "mdi:tools",
    description: (
      <>
        Design, verify, and tapeout your chip with a tool suite of your choice
        using Substrate's extensible plugin system.
      </>
    ),
  },
  {
    title: "100% Performant Rust Code",
    icon: "mdi:code-block-tags",
    description: (
      <>
        Generators can be written entirely in Rust, providing high performance
        alongside memory safety and type checking.
      </>
    ),
  },
  {
    title: "Open Source",
    icon: "mdi:web",
    description: (
      <>
        The core of Substrate is open source, meaning anyone can write a circuit
        generator without an expensive license.
      </>
    ),
  },
];

function Feature({ title, icon, description }: FeatureItem) {
  return (
    <div className={clsx("col col--4")}>
      <div className="text--center">
        <Heading as="h3">
          <Icon icon={icon} height="50" />
        </Heading>
      </div>
      <div className="text--center padding-horiz--md">
        <Heading as="h3">{title}</Heading>
        <p>{description}</p>
      </div>
    </div>
  );
}

export default function HomepageFeatures(): JSX.Element {
  return (
    <section className={styles.features}>
      <div className="container padding-vert--lg">
        <div className="row">
          {FeatureList.map((props, idx) => (
            <Feature key={idx} {...props} />
          ))}
        </div>
      </div>
    </section>
  );
}
