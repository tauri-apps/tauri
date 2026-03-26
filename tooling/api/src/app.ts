
import java.time.LocalDateTime;
import java.time.format.DateTimeFormatter;
import java.util.*;

// Patient class to store patient information
class Patient {
    private String patientId;
    private String name;
    private String phone;
    private String email;
    private String medicalHistory;
    private List<Appointment> appointments;

    public Patient(String patientId, String name, String phone, String email) {
        this.patientId = patientId;
        this.name = name;
        this.phone = phone;
        this.email = email;
        this.medicalHistory = "";
        this.appointments = new ArrayList<>();
    }

    // Getters and setters
    public String getPatientId() { return patientId; }
    public String getName() { return name; }
    public String getPhone() { return phone; }
    public String getEmail() { return email; }
    public String getMedicalHistory() { return medicalHistory; }
    public void setMedicalHistory(String medicalHistory) { this.medicalHistory = medicalHistory; }
    public List<Appointment> getAppointments() { return appointments; }

    @Override
    public String toString() {
        return String.format("ID: %s | Name: %s | Phone: %s | Email: %s", 
                           patientId, name, phone, email);
    }
}

// Appointment class
class Appointment {
    private String appointmentId;
    private String patientId;
    private LocalDateTime dateTime;
    private String doctorName;
    private String status; // "Scheduled", "Completed", "Cancelled"

    public Appointment(String appointmentId, String patientId, LocalDateTime dateTime, 
                      String doctorName) {
        this.appointmentId = appointmentId;
        this.patientId = patientId;
        this.dateTime = dateTime;
        this.doctorName = doctorName;
        this.status = "Scheduled";
    }

    // Getters
    public String getAppointmentId() { return appointmentId; }
    public String getPatientId() { return patientId; }
    public LocalDateTime getDateTime() { return dateTime; }
    public String getDoctorName() { return doctorName; }
    public String getStatus() { return status; }
    public void setStatus(String status) { this.status = status; }

    @Override
    public String toString() {
        DateTimeFormatter formatter = DateTimeFormatter.ofPattern("yyyy-MM-dd HH:mm");
        return String.format("Appt ID: %s | Patient: %s | Doctor: %s | Time: %s | Status: %s",
                           appointmentId, patientId, doctorName, 
                           dateTime.format(formatter), status);
    }
}

// Main Medical Clinic Management System
public class MedicalClinicSystem {
    private static Map<String, Patient> patients = new HashMap<>();
    private static List<Appointment> appointments = new ArrayList<>();
    private static int patientCounter = 1000;
    private static int appointmentCounter = 2000;
    private static Scanner scanner = new Scanner(System.in);

    public static void main(String[] args) {
        System.out.println("=== MEDICAL CLINIC MANAGEMENT SYSTEM ===");
        showMenu();
    }

    private static void showMenu() {
        while (true) {
            System.out.println("\n=== MAIN MENU ===");
            System.out.println("1. Register New Patient");
            System.out.println("2. View All Patients");
            System.out.println("3. Schedule Appointment");
            System.out.println("4. View All Appointments");
            System.out.println("5. Update Patient Medical History");
            System.out.println("6. Cancel Appointment");
            System.out.println("7. Search Patient by ID");
            System.out.println("0. Exit");
            
            System.out.print("Enter your choice: ");
            int choice = scanner.nextInt();
            scanner.nextLine(); // consume newline

            switch (choice) {
                case 1: registerPatient(); break;
                case 2: viewAllPatients(); break;
                case 3: scheduleAppointment(); break;
                case 4: viewAllAppointments(); break;
                case 5: updateMedicalHistory(); break;
                case 6: cancelAppointment(); break;
                case 7: searchPatient(); break;
                case 0: System.out.println("Thank you for using Medical Clinic System!"); return;
                default: System.out.println("Invalid choice! Try again.");
            }
        }
    }

    private static void registerPatient() {
        System.out.print("Enter patient name: ");
        String name = scanner.nextLine();
        System.out.print("Enter phone number: ");
        String phone = scanner.nextLine();
        System.out.print("Enter email: ");
        String email = scanner.nextLine();

        String patientId = "P" + (++patientCounter);
        Patient patient = new Patient(patientId, name, phone, email);
        patients.put(patientId, patient);

        System.out.println("✅ Patient registered successfully!");
        System.out.println("Patient ID: " + patientId);
        System.out.println(patient);
    }

    private static void viewAllPatients() {
        if (patients.isEmpty()) {
            System.out.println("No patients registered yet.");
            return;
        }

        System.out.println("\n=== ALL PATIENTS ===");
        patients.values().forEach(System.out::println);
    }

    private static void scheduleAppointment() {
        System.out.print("Enter patient ID: ");
        String patientId = scanner.nextLine();

        Patient patient = patients.get(patientId);
        if (patient == null) {
            System.out.println("❌ Patient not found!");
            return;
        }

        System.out.print("Enter appointment date (yyyy-MM-dd HH:mm): ");
        String dateTimeStr = scanner.nextLine();
        DateTimeFormatter formatter = DateTimeFormatter.ofPattern("yyyy-MM-dd HH:mm");
        LocalDateTime dateTime;
        try {
            dateTime = LocalDateTime.parse(dateTimeStr, formatter);
        } catch (Exception e) {
            System.out.println("❌ Invalid date format!");
            return;
        }

        System.out.print("Enter doctor name: ");
        String doctorName = scanner.nextLine();

        String appointmentId = "A" + (++appointmentCounter);
        Appointment appointment = new Appointment(appointmentId, patientId, dateTime, doctorName);
        appointments.add(appointment);
        patient.getAppointments().add(appointment);

        System.out.println("✅ Appointment scheduled successfully!");
        System.out.println(appointment);
    }

    private static void viewAllAppointments() {
        if (appointments.isEmpty()) {
            System.out.println("No appointments scheduled yet.");
            return;
        }

        System.out.println("\n=== ALL APPOINTMENTS ===");
        appointments.forEach(System.out::println);
    }

    private static void updateMedicalHistory() {
        System.out.print("Enter patient ID: ");
        String patientId = scanner.nextLine();

        Patient patient = patients.get(patientId);
        if (patient == null) {
            System.out.println("❌ Patient not found!");
            return;
        }

        System.out.println("Current Medical History: " + patient.getMedicalHistory());
        System.out.print("Enter new medical history: ");
        String history = scanner.nextLine();
        patient.setMedicalHistory(history);

        System.out.println("✅ Medical history updated successfully!");
    }

    private static void cancelAppointment() {
        System.out.print("Enter appointment ID: ");
        String appointmentId = scanner.nextLine();

        Optional<Appointment> appointment = appointments.stream()
            .filter(a -> a.getAppointmentId().equals(appointmentId))
            .findFirst();

        if (appointment.isPresent()) {
            appointment.get().setStatus("Cancelled");
            System.out.println("✅ Appointment cancelled successfully!");
            System.out.println(appointment.get());
        } else {
            System.out.println("❌ Appointment not found!");
        }
    }

    private static void searchPatient() {
        System.out.print("Enter patient ID: ");
        String patientId = scanner.nextLine();

        Patient patient = patients.get(patientId);
        if (patient == null) {
            System.out.println("❌ Patient not found!");
            return;
        }

        System.out.println("\n=== PATIENT DETAILS ===");
        System.out.println(patient);
        System.out.println("Medical History: " + patient.getMedicalHistory());
        System.out.println("\nAppointments:");
        patient.getAppointments().forEach(System.out::println);
    }
}
