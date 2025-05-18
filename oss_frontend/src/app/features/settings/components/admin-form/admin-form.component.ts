import {Component, OnInit} from '@angular/core';
import {FormBuilder, FormGroup, ReactiveFormsModule, Validators} from '@angular/forms';
import {ActivatedRoute, Router} from '@angular/router';
import {AdminService} from '../../../../shared/services/admin.service';
import {AdminGroup} from '../../../../shared/models/admin.model';
import {PaginationParams} from '../../../../shared/models/pagination.model';

@Component({
  selector: 'app-admin-form',
  templateUrl: './admin-form.component.html',
  imports: [
    ReactiveFormsModule,
  ],
  styleUrls: ['./admin-form.component.scss'],
})
export class AdminFormComponent implements OnInit {
  adminForm: FormGroup;
  adminGroups: AdminGroup[] = [];
  isEditMode = false;
  loading = false;
  submitting = false;
  error = '';
  adminId?: number;

  constructor(
    private fb: FormBuilder,
    private adminService: AdminService,
    private router: Router,
    private route: ActivatedRoute,
  ) {
    this.adminForm = this.fb.group({
      email: ['', [Validators.required, Validators.email]],
      first_name: ['', [Validators.required, Validators.maxLength(150)]],
      last_name: ['', [Validators.required, Validators.maxLength(150)]],
      position: ['', Validators.maxLength(100)],
      phone_number: ['', Validators.maxLength(20)],
      is_active: [true],
      is_staff: [false],
      is_superuser: [false],
      groups: [[]],
    });
  }

  ngOnInit(): void {
    this.adminId = this.route.snapshot.params['id'];
    this.loadAdminGroups();

    if (this.adminId) {
      this.isEditMode = true;
      this.loadAdmin();
    }
  }

  loadAdminGroups(): void {
    const params: PaginationParams = {
      page: 1,
      page_size: 100, // Load a reasonable number of groups
    };

    this.adminService.getAllAdminGroups(params)
      .subscribe({
        next: (response) => {
          this.adminGroups = response.results;
        },
        error: (err) => {
          this.error = 'Failed to load admin groups. Please try again later.';
          console.error('Error loading admin groups:', err);
        },
      });
  }

  loadAdmin(): void {
    if (!this.adminId) return;

    this.loading = true;
    this.adminService.getAdmin(this.adminId)
      .subscribe({
        next: (admin) => {
          this.adminForm.patchValue(admin);
          this.loading = false;
        },
        error: (err) => {
          this.error = 'Failed to load admin. Please try again later.';
          console.error('Error loading admin:', err);
          this.loading = false;
        },
      });
  }

  onGroupChange(event: Event, groupId: number): void {
    const checkbox = event.target as HTMLInputElement;
    const currentGroups = this.adminForm.get('groups')?.value || [];

    if (checkbox.checked) {
      this.adminForm.patchValue({
        groups: [...currentGroups, groupId],
      });
    } else {
      this.adminForm.patchValue({
        groups: currentGroups.filter((id: number) => id !== groupId),
      });
    }
  }

  isGroupSelected(groupId: number): boolean {
    const selectedGroups = this.adminForm.get('groups')?.value || [];
    return selectedGroups.includes(groupId);
  }

  onSubmit(): void {
    if (this.adminForm.invalid) {
      return;
    }

    this.submitting = true;
    const adminData = this.adminForm.value;

    const request = this.isEditMode
      ? this.adminService.updateAdmin(this.adminId!, adminData)
      : this.adminService.createAdmin(adminData);

    request.subscribe({
      next: () => {
        this.router.navigate(['../'], {relativeTo: this.route});
      },
      error: (err) => {
        this.error = `Failed to ${this.isEditMode ? 'update' : 'create'} admin. Please try again later.`;
        console.error(`Error ${this.isEditMode ? 'updating' : 'creating'} admin:`, err);
        this.submitting = false;
      },
    });
  }

  cancel(): void {
    this.router.navigate(['../'], {relativeTo: this.route});
  }
}
