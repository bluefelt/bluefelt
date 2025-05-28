import { Link } from 'react-router-dom';

interface BreadcrumbsProps {
  items: {
    label: string;
    href?: string;
  }[];
}

export default function Breadcrumbs({ items }: BreadcrumbsProps) {
  return (
    <div style={styles.breadcrumbs}>
      {items.map((item, index) => (
        <span key={index}>
          {item.href ? (
            <Link to={item.href} style={styles.breadcrumbsLink}>
              {item.label}
            </Link>
          ) : (
            <span>{item.label}</span>
          )}
          {index < items.length - 1 && (
            <span>{' > '}</span>
          )}
        </span>
      ))}
    </div>
  );
}

const styles = {
  breadcrumbs: {
    margin: "10px",
    fontFamily: "Roboto Condensed, sans-serif",
    fontSize: "16pt",
    color: "#D8B260",
  },
  breadcrumbsLink: {
    textDecoration: "underline",
  },
};